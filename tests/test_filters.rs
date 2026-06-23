use visor::{BBox, InventoryModeFilter, Latitude, ListingsFilter, ListingsFilterBase, Longitude};

// Phase 3.6 types — always available, used in unconditional tests.
use visor::{
    FacetField, FacetMetric, FacetMetricAggregate, FacetMetricMeasure, FacetSort, FacetsFilter,
    VisorError,
};

#[cfg(phase_contracts)]
use uuid::Uuid;

#[cfg(phase_contracts)]
use visor::{
    AvailabilityStatus, DealerFilter, DealerType, GeoFilter, GeoOrigin, HistoryKeyword,
    InventoryType, ListingField, ListingInclude, PostalCode, RadiusMiles, SortOrder, StateCode,
    VinPattern,
};

fn has_key(params: &[(String, String)], key: &str) -> bool {
    params.iter().any(|(k, _)| k == key)
}

#[cfg(phase_contracts)]
fn param(params: &[(String, String)], key: &str) -> Option<String> {
    params
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
}

// ── BBox validation (unconditional — tests the constructor, not to_params) ───

#[test]
fn bbox_new_valid_box() {
    let result = BBox::new(
        Longitude::new(-122.5).unwrap(),
        Latitude::new(37.2).unwrap(),
        Longitude::new(-121.9).unwrap(),
        Latitude::new(37.8).unwrap(),
    );
    assert!(result.is_ok());
}

#[test]
fn bbox_new_rejects_inverted_south_north() {
    let result = BBox::new(
        Longitude::new(-100.0).unwrap(),
        Latitude::new(40.0).unwrap(), // south
        Longitude::new(-90.0).unwrap(),
        Latitude::new(35.0).unwrap(), // north < south
    );
    assert!(matches!(result, Err(VisorError::InvalidFilter { .. })));
}

#[test]
fn bbox_new_equal_south_north_is_valid() {
    // south == north is a degenerate box but not forbidden
    let result = BBox::new(
        Longitude::new(-100.0).unwrap(),
        Latitude::new(40.0).unwrap(),
        Longitude::new(-90.0).unwrap(),
        Latitude::new(40.0).unwrap(),
    );
    assert!(result.is_ok());
}

#[test]
fn bbox_new_rejects_diagonal_over_1000_miles() {
    // Roughly continental US — ~3600 mile diagonal
    let result = BBox::new(
        Longitude::new(-125.0).unwrap(),
        Latitude::new(25.0).unwrap(),
        Longitude::new(-66.0).unwrap(),
        Latitude::new(49.0).unwrap(),
    );
    assert!(matches!(result, Err(VisorError::InvalidFilter { .. })));
}

#[test]
fn bbox_new_antimeridian_crossing_accepted() {
    // west=175 > east=-175: longitude span is 10°, diagonal well under 1000 miles
    let result = BBox::new(
        Longitude::new(175.0).unwrap(),
        Latitude::new(50.0).unwrap(),
        Longitude::new(-175.0).unwrap(),
        Latitude::new(55.0).unwrap(),
    );
    assert!(result.is_ok());
}

// ── Serialization golden tests ────────────────────────────────────────────────

#[cfg(phase_contracts)]
#[test]
fn default_listings_filter_emits_limit_offset_sort() {
    let params = ListingsFilter::default().to_params();
    assert_eq!(param(&params, "limit").as_deref(), Some("50"));
    assert_eq!(param(&params, "offset").as_deref(), Some("0"));
    assert_eq!(param(&params, "sort").as_deref(), Some("days_on_market"));
}

#[test]
fn inventory_mode_active_omitted_from_params() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            inventory_mode: InventoryModeFilter::Active,
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert!(!has_key(&filter.to_params(), "inventory_status"));
}

#[cfg(phase_contracts)]
#[test]
fn inventory_mode_sold_emitted_as_wire_value() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            inventory_mode: InventoryModeFilter::Sold {
                sold_within_days: Some(30),
            },
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    let params = filter.to_params();
    assert_eq!(param(&params, "inventory_status").as_deref(), Some("sold"));
    assert_eq!(param(&params, "sold_within_days").as_deref(), Some("30"));
}

#[cfg(phase_contracts)]
#[test]
fn inventory_mode_sold_without_window_emits_sold_but_no_days() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            inventory_mode: InventoryModeFilter::Sold {
                sold_within_days: None,
            },
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    let params = filter.to_params();
    assert_eq!(param(&params, "inventory_status").as_deref(), Some("sold"));
    assert!(!has_key(&params, "sold_within_days"));
}

#[cfg(phase_contracts)]
#[test]
fn assembly_location_uses_pipe_separator() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            assembly_location: Some(vec!["US".to_string(), "MX".to_string()]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "assembly_location").as_deref(),
        Some("US|MX")
    );
}

#[cfg(phase_contracts)]
#[test]
fn exclude_assembly_location_uses_plus_separator() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            exclude_assembly_location: Some(vec!["KR".to_string(), "JP".to_string()]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "exclude_assembly_location").as_deref(),
        Some("KR+JP")
    );
}

#[cfg(phase_contracts)]
#[test]
fn bbox_serialized_as_west_south_east_north() {
    let bbox = BBox::new(
        Longitude::new(-122.5).unwrap(),
        Latitude::new(37.2).unwrap(),
        Longitude::new(-121.9).unwrap(),
        Latitude::new(37.8).unwrap(),
    )
    .unwrap();
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            geo: Some(GeoFilter::BBox(bbox)),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "bbox").as_deref(),
        Some("-122.5,37.2,-121.9,37.8")
    );
}

#[cfg(phase_contracts)]
#[test]
fn antimeridian_bbox_serialized_correctly() {
    let bbox = BBox::new(
        Longitude::new(175.0).unwrap(),
        Latitude::new(50.0).unwrap(),
        Longitude::new(-175.0).unwrap(),
        Latitude::new(55.0).unwrap(),
    )
    .unwrap();
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            geo: Some(GeoFilter::BBox(bbox)),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "bbox").as_deref(),
        Some("175,50,-175,55")
    );
}

#[cfg(phase_contracts)]
#[test]
fn geo_origin_postal_code_emitted_as_postal_code_param() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            geo: Some(GeoFilter::Origin(GeoOrigin::PostalCode(
                PostalCode::new("90210").unwrap(),
            ))),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    let params = filter.to_params();
    assert_eq!(param(&params, "postal_code").as_deref(), Some("90210"));
    assert!(!has_key(&params, "latitude"));
    assert!(!has_key(&params, "longitude"));
}

#[cfg(phase_contracts)]
#[test]
fn geo_origin_coordinates_emitted_as_lat_lon() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            geo: Some(GeoFilter::Origin(GeoOrigin::Coordinates {
                latitude: Latitude::new(34.05).unwrap(),
                longitude: Longitude::new(-118.25).unwrap(),
            })),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    let params = filter.to_params();
    assert_eq!(param(&params, "latitude").as_deref(), Some("34.05"));
    assert_eq!(param(&params, "longitude").as_deref(), Some("-118.25"));
    assert!(!has_key(&params, "postal_code"));
}

#[cfg(phase_contracts)]
#[test]
fn geo_radius_with_postal_origin_emits_radius_and_postal_code() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            geo: Some(GeoFilter::Radius {
                origin: GeoOrigin::PostalCode(PostalCode::new("90210").unwrap()),
                miles: RadiusMiles::new(25.0).unwrap(),
            }),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    let params = filter.to_params();
    assert_eq!(param(&params, "postal_code").as_deref(), Some("90210"));
    assert_eq!(param(&params, "radius").as_deref(), Some("25"));
    assert!(!has_key(&params, "latitude"));
    assert!(!has_key(&params, "bbox"));
}

#[cfg(phase_contracts)]
#[test]
fn geo_radius_with_coordinates_emits_radius_lat_lon() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            geo: Some(GeoFilter::Radius {
                origin: GeoOrigin::Coordinates {
                    latitude: Latitude::new(34.05).unwrap(),
                    longitude: Longitude::new(-118.25).unwrap(),
                },
                miles: RadiusMiles::new(50.0).unwrap(),
            }),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    let params = filter.to_params();
    assert_eq!(param(&params, "latitude").as_deref(), Some("34.05"));
    assert_eq!(param(&params, "longitude").as_deref(), Some("-118.25"));
    assert_eq!(param(&params, "radius").as_deref(), Some("50"));
    assert!(!has_key(&params, "bbox"));
}

#[cfg(phase_contracts)]
#[test]
fn sort_wire_values_match_python_sdk() {
    let cases = [
        (SortOrder::DaysOnMarket, "days_on_market"),
        (SortOrder::DaysOnMarketDesc, "-days_on_market"),
        (SortOrder::Price, "price"),
        (SortOrder::PriceDesc, "-price"),
        (SortOrder::Miles, "miles"),
        (SortOrder::MilesDesc, "-miles"),
        (SortOrder::Msrp, "msrp"),
        (SortOrder::MsrpDesc, "-msrp"),
        (SortOrder::Discount, "discount"),
        (SortOrder::DiscountDesc, "-discount"),
        (SortOrder::Distance, "distance"),
    ];
    for (sort, expected) in cases {
        let params = ListingsFilter {
            sort,
            ..ListingsFilter::default()
        }
        .to_params();
        assert_eq!(
            param(&params, "sort").as_deref(),
            Some(expected),
            "wrong wire value for sort variant"
        );
    }
}

#[cfg(phase_contracts)]
#[test]
fn fields_projection_emitted_comma_separated() {
    let filter = ListingsFilter {
        fields: Some(vec![
            ListingField::Make,
            ListingField::Price,
            ListingField::Year,
        ]),
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "fields").as_deref(),
        Some("make,price,year")
    );
}

#[cfg(phase_contracts)]
#[test]
fn include_emitted_comma_separated() {
    let filter = ListingsFilter {
        include: Some(vec![ListingInclude::PriceHistory, ListingInclude::Options]),
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "include").as_deref(),
        Some("price_history,options")
    );
}

#[cfg(phase_contracts)]
#[test]
fn snapshot_date_serialized_as_iso8601() {
    use chrono::NaiveDate;
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            inventory_mode: InventoryModeFilter::Snapshot {
                date: NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            },
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "snapshot_date").as_deref(),
        Some("2024-03-15")
    );
}

#[cfg(phase_contracts)]
#[test]
fn comma_separated_list_fields_join_correctly() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            make: Some(vec!["Toyota".to_string(), "Honda".to_string()]),
            state: Some(vec![
                StateCode::new("CA").unwrap(),
                StateCode::new("TX").unwrap(),
            ]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    let params = filter.to_params();
    assert_eq!(param(&params, "make").as_deref(), Some("Toyota,Honda"));
    assert_eq!(param(&params, "state").as_deref(), Some("CA,TX"));
}

#[cfg(phase_contracts)]
#[test]
fn inventory_type_wire_values_emitted() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            inventory_type: Some(vec![InventoryType::New, InventoryType::Certified]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "inventory_type").as_deref(),
        Some("new,certified")
    );
}

#[cfg(phase_contracts)]
#[test]
fn history_keywords_emitted_comma_separated() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            keywords: Some(vec![HistoryKeyword::OneOwner, HistoryKeyword::CleanTitle]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "keywords").as_deref(),
        Some("one_owner,clean_title")
    );
}

#[cfg(phase_contracts)]
#[test]
fn numeric_range_params_emitted() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            min_price: Some(10_000),
            max_price: Some(50_000),
            min_mileage: Some(0),
            max_mileage: Some(30_000),
            min_msrp: Some(25_000),
            max_msrp: Some(80_000),
            min_days_on_market: Some(1),
            max_days_on_market: Some(90),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    let params = filter.to_params();
    assert_eq!(param(&params, "min_price").as_deref(), Some("10000"));
    assert_eq!(param(&params, "max_price").as_deref(), Some("50000"));
    assert_eq!(param(&params, "min_mileage").as_deref(), Some("0"));
    assert_eq!(param(&params, "max_mileage").as_deref(), Some("30000"));
    assert_eq!(param(&params, "min_msrp").as_deref(), Some("25000"));
    assert_eq!(param(&params, "max_msrp").as_deref(), Some("80000"));
    assert_eq!(param(&params, "min_days_on_market").as_deref(), Some("1"));
    assert_eq!(param(&params, "max_days_on_market").as_deref(), Some("90"));
}

#[cfg(phase_contracts)]
#[test]
fn vin_pattern_emitted_comma_separated() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            vin_pattern: Some(vec![
                VinPattern::new("1HGCM826*").unwrap(),
                VinPattern::new("5FNR?*").unwrap(),
            ]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "vin_pattern").as_deref(),
        Some("1HGCM826*,5FNR?*")
    );
}

#[cfg(phase_contracts)]
#[test]
fn dealer_id_serialized_as_hyphenated_uuid() {
    let id = Uuid::from_u128(0x12345678_1234_1234_1234_123456789abc);
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            dealer_id: Some(vec![id]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    let params = filter.to_params();
    // UUID should be hyphenated (8-4-4-4-12 format)
    assert_eq!(
        param(&params, "dealer_id").as_deref(),
        Some("12345678-1234-1234-1234-123456789abc")
    );
}

#[cfg(phase_contracts)]
#[test]
fn availability_status_emitted_comma_separated() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            availability_status: Some(vec![AvailabilityStatus::Stock, AvailabilityStatus::Transit]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "availability_status").as_deref(),
        Some("stock,transit")
    );
}

#[cfg(phase_contracts)]
#[test]
fn empty_vec_is_omitted_from_params() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            make: Some(vec![]),
            model: Some(vec![]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    let params = filter.to_params();
    assert!(!has_key(&params, "make"));
    assert!(!has_key(&params, "model"));
}

#[cfg(phase_contracts)]
#[test]
fn year_emitted_as_comma_separated_integers() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            year: Some(vec![2022, 2023, 2024]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "year").as_deref(),
        Some("2022,2023,2024")
    );
}

#[cfg(phase_contracts)]
#[test]
fn seating_capacity_and_doors_emitted() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            seating_capacity: Some(vec![5, 7]),
            cylinders: Some(vec![0, 4, 6]),
            doors: Some(vec![2, 4]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    let params = filter.to_params();
    assert_eq!(param(&params, "seating_capacity").as_deref(), Some("5,7"));
    assert_eq!(param(&params, "cylinders").as_deref(), Some("0,4,6"));
    assert_eq!(param(&params, "doors").as_deref(), Some("2,4"));
}

#[cfg(phase_contracts)]
#[test]
fn dealer_type_emitted_comma_separated() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            dealer_type: Some(vec![DealerType::Franchise, DealerType::Independent]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "dealer_type").as_deref(),
        Some("franchise,independent")
    );
}

#[cfg(phase_contracts)]
#[test]
fn exclusion_filters_emitted_with_correct_separators() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            exclude_make: Some(vec!["Ford".to_string(), "GM".to_string()]),
            exclude_year: Some(vec![2019, 2020]),
            exclude_state: Some(vec![StateCode::new("AK").unwrap()]),
            exclude_inventory_type: Some(vec![InventoryType::Used]),
            exclude_assembly_location: Some(vec!["KR".to_string(), "JP".to_string()]),
            exclude_assembly_country: Some(vec!["CN".to_string()]),
            exclude_keywords: Some(vec![HistoryKeyword::Fleet]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    let params = filter.to_params();
    assert_eq!(param(&params, "exclude_make").as_deref(), Some("Ford,GM"));
    assert_eq!(param(&params, "exclude_year").as_deref(), Some("2019,2020"));
    assert_eq!(param(&params, "exclude_state").as_deref(), Some("AK"));
    assert_eq!(
        param(&params, "exclude_inventory_type").as_deref(),
        Some("used")
    );
    assert_eq!(
        param(&params, "exclude_assembly_location").as_deref(),
        Some("KR+JP")
    );
    assert_eq!(
        param(&params, "exclude_assembly_country").as_deref(),
        Some("CN")
    );
    assert_eq!(param(&params, "exclude_keywords").as_deref(), Some("fleet"));
}

// ── FacetsFilter serialization ────────────────────────────────────────────────

#[cfg(phase_contracts)]
#[test]
fn facets_filter_always_emits_sort() {
    let filter = FacetsFilter::new(vec![FacetField::Make]);
    let params = filter.to_params();
    assert!(
        has_key(&params, "sort"),
        "FacetsFilter must always emit sort"
    );
    assert_eq!(param(&params, "sort").as_deref(), Some("-count"));
}

#[cfg(phase_contracts)]
#[test]
fn facets_filter_always_emits_facets() {
    let filter = FacetsFilter::new(vec![FacetField::Make, FacetField::Model]);
    let params = filter.to_params();
    assert_eq!(param(&params, "facets").as_deref(), Some("make,model"));
}

#[cfg(phase_contracts)]
#[test]
fn facets_filter_omits_metric_when_none() {
    let filter = FacetsFilter::new(vec![FacetField::Make]);
    assert!(!has_key(&filter.to_params(), "metric"));
}

#[cfg(phase_contracts)]
#[test]
fn facets_filter_emits_metric_count() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make]);
    filter.metric = Some(FacetMetric::Count);
    assert_eq!(
        param(&filter.to_params(), "metric").as_deref(),
        Some("count")
    );
}

#[cfg(phase_contracts)]
#[test]
fn facets_filter_emits_aggregate_metric() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make]);
    filter.metric = Some(FacetMetric::Aggregate {
        measure: FacetMetricMeasure::Price,
        aggregate: FacetMetricAggregate::P95,
    });
    assert_eq!(
        param(&filter.to_params(), "metric").as_deref(),
        Some("price.p95")
    );
}

#[cfg(phase_contracts)]
#[test]
fn facets_filter_omits_facet_value_limit_when_none() {
    let filter = FacetsFilter::new(vec![FacetField::Make]);
    assert!(!has_key(&filter.to_params(), "facet_value_limit"));
}

#[cfg(phase_contracts)]
#[test]
fn facets_filter_emits_facet_value_limit_when_set() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make]);
    filter.facet_value_limit = Some(50);
    assert_eq!(
        param(&filter.to_params(), "facet_value_limit").as_deref(),
        Some("50")
    );
}

#[cfg(phase_contracts)]
#[test]
fn facet_sort_wire_values() {
    let cases = [
        (FacetSort::Count, "count"),
        (FacetSort::CountDesc, "-count"),
        (FacetSort::Metric, "metric"),
        (FacetSort::MetricDesc, "-metric"),
    ];
    for (sort, expected) in cases {
        let mut filter = FacetsFilter::new(vec![FacetField::Make]);
        filter.sort = sort;
        assert_eq!(
            param(&filter.to_params(), "sort").as_deref(),
            Some(expected),
            "wrong wire value for FacetSort variant"
        );
    }
}

#[cfg(phase_contracts)]
#[test]
fn facets_filter_composes_with_base_params() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make]);
    filter.base.make = Some(vec!["Toyota".to_string()]);
    filter.base.min_price = Some(20_000);
    let params = filter.to_params();
    // facets come first
    assert_eq!(param(&params, "facets").as_deref(), Some("make"));
    // base params are included
    assert_eq!(param(&params, "make").as_deref(), Some("Toyota"));
    assert_eq!(param(&params, "min_price").as_deref(), Some("20000"));
}

#[cfg(phase_contracts)]
#[test]
fn facets_filter_empty_base_vecs_are_omitted() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make]);
    filter.base.model = Some(vec![]);
    let params = filter.to_params();
    assert!(!has_key(&params, "model"));
}

// ── DealerFilter serialization ────────────────────────────────────────────────

#[cfg(phase_contracts)]
#[test]
fn dealer_filter_type_field_uses_wire_key_type() {
    let filter = DealerFilter {
        dealer_type: Some(DealerType::Franchise),
        ..DealerFilter::default()
    };
    let params = filter.to_params();
    assert_eq!(param(&params, "type").as_deref(), Some("franchise"));
    assert!(
        !has_key(&params, "dealer_type"),
        "wire key must be 'type', not 'dealer_type'"
    );
}

#[cfg(phase_contracts)]
#[test]
fn dealer_filter_independent_wire_value() {
    let filter = DealerFilter {
        dealer_type: Some(DealerType::Independent),
        ..DealerFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "type").as_deref(),
        Some("independent")
    );
}

#[cfg(phase_contracts)]
#[test]
fn dealer_filter_default_emits_limit_and_offset() {
    let params = DealerFilter::default().to_params();
    assert_eq!(param(&params, "limit").as_deref(), Some("50"));
    assert_eq!(param(&params, "offset").as_deref(), Some("0"));
}

#[cfg(phase_contracts)]
#[test]
fn dealer_filter_state_emitted_comma_separated() {
    let filter = DealerFilter {
        state: Some(vec![
            StateCode::new("CA").unwrap(),
            StateCode::new("TX").unwrap(),
        ]),
        ..DealerFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "state").as_deref(),
        Some("CA,TX")
    );
}

#[cfg(phase_contracts)]
#[test]
fn dealer_filter_make_emitted_comma_separated() {
    let filter = DealerFilter {
        make: Some(vec!["Toyota".to_string(), "Honda".to_string()]),
        ..DealerFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "make").as_deref(),
        Some("Toyota,Honda")
    );
}

#[cfg(phase_contracts)]
#[test]
fn dealer_filter_q_emitted() {
    let filter = DealerFilter {
        q: Some("claremont".to_string()),
        ..DealerFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "q").as_deref(),
        Some("claremont")
    );
}

#[cfg(phase_contracts)]
#[test]
fn dealer_filter_dealer_id_serialized_as_hyphenated_uuid() {
    let id = Uuid::from_u128(0x12345678_1234_1234_1234_123456789abc);
    let filter = DealerFilter {
        dealer_id: Some(vec![id]),
        ..DealerFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "dealer_id").as_deref(),
        Some("12345678-1234-1234-1234-123456789abc")
    );
}

// ── Validation error tests ────────────────────────────────────────────────────

// NOTE: Several geo/inventory-mode validation tests that existed in earlier
// drafts are intentionally absent here. Phase 3.5 introduces `GeoFilter` and
// `InventoryModeFilter` as enums, making the following combinations impossible
// to construct at the type level:
//   - radius without an anchor (GeoFilter::Radius requires an origin)
//   - radius with both anchors (GeoOrigin is one-of postal-code or lat/lon)
//   - lat without lon (GeoOrigin::Coordinates requires both)
//   - bbox + radius together (GeoFilter is an enum; only one variant applies)
//   - sold_within_days without sold mode (only exists on InventoryModeFilter::Sold)
//   - snapshot_date with sold mode (only exists on InventoryModeFilter::Snapshot)
//   - sold_within_days + snapshot_date together (impossible across two variants)
// These invariants are now compile-time guarantees, not runtime checks.

#[cfg(phase_contracts)]
fn assert_invalid_filter(result: Result<(), VisorError>) {
    assert!(
        matches!(result, Err(VisorError::InvalidFilter { .. })),
        "expected InvalidFilter, got: {result:?}"
    );
}

#[cfg(phase_contracts)]
#[test]
fn listings_limit_over_100_is_invalid() {
    let filter = ListingsFilter {
        limit: 101,
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(phase_contracts)]
#[test]
fn listings_limit_100_is_valid() {
    let filter = ListingsFilter {
        limit: 100,
        ..ListingsFilter::default()
    };
    assert!(filter.validate().is_ok());
}

#[cfg(phase_contracts)]
#[test]
fn listings_base_dealer_id_over_50_is_invalid() {
    let ids: Vec<Uuid> = (0u128..51).map(Uuid::from_u128).collect();
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            dealer_id: Some(ids),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(phase_contracts)]
#[test]
fn listings_base_vin_pattern_over_10_is_invalid() {
    let patterns = (0..11)
        .map(|_| VinPattern::new("1HGCM826*").unwrap())
        .collect();
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            vin_pattern: Some(patterns),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(phase_contracts)]
#[test]
fn listings_numeric_range_min_greater_than_max_is_invalid() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            min_price: Some(50_000),
            max_price: Some(20_000),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(phase_contracts)]
#[test]
fn listings_mileage_range_validated() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            min_mileage: Some(30_000),
            max_mileage: Some(10_000),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(phase_contracts)]
#[test]
fn listings_seating_capacity_zero_is_invalid() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            seating_capacity: Some(vec![0]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(phase_contracts)]
#[test]
fn listings_doors_zero_is_invalid() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            doors: Some(vec![0]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(phase_contracts)]
#[test]
fn listings_cylinders_zero_is_valid() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            cylinders: Some(vec![0]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert!(filter.validate().is_ok());
}

#[cfg(phase_contracts)]
#[test]
fn listings_sold_within_days_zero_is_invalid() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            inventory_mode: InventoryModeFilter::Sold {
                sold_within_days: Some(0),
            },
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(phase_contracts)]
#[test]
fn listings_blank_make_element_is_invalid() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            make: Some(vec!["Toyota".to_string(), "  ".to_string()]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(phase_contracts)]
#[test]
fn listings_empty_make_element_is_invalid() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            make: Some(vec!["".to_string()]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(phase_contracts)]
#[test]
fn listings_sort_distance_without_geo_origin_is_invalid() {
    let filter = ListingsFilter {
        sort: SortOrder::Distance,
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(phase_contracts)]
#[test]
fn listings_sort_distance_with_bbox_is_invalid() {
    let bbox = BBox::new(
        Longitude::new(-122.5).unwrap(),
        Latitude::new(37.2).unwrap(),
        Longitude::new(-121.9).unwrap(),
        Latitude::new(37.8).unwrap(),
    )
    .unwrap();
    let filter = ListingsFilter {
        sort: SortOrder::Distance,
        base: ListingsFilterBase {
            geo: Some(GeoFilter::BBox(bbox)),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(phase_contracts)]
#[test]
fn listings_sort_distance_with_origin_is_valid() {
    let filter = ListingsFilter {
        sort: SortOrder::Distance,
        base: ListingsFilterBase {
            geo: Some(GeoFilter::Origin(GeoOrigin::PostalCode(
                PostalCode::new("90210").unwrap(),
            ))),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert!(filter.validate().is_ok());
}

#[cfg(phase_contracts)]
#[test]
fn facets_filter_empty_facets_is_invalid() {
    assert_invalid_filter(FacetsFilter::new(vec![]).validate());
}

#[cfg(phase_contracts)]
#[test]
fn facets_base_validation_propagated() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make]);
    filter.base.min_price = Some(50_000);
    filter.base.max_price = Some(20_000);
    assert_invalid_filter(filter.validate());
}

#[cfg(phase_contracts)]
#[test]
fn dealer_filter_over_100_ids_is_invalid() {
    // dealer_id takes Vec<Uuid>; generate 101 deterministic nil-variant UUIDs.
    // DealerFilter allows up to 100 IDs (search-dealers.md line 66).
    let ids: Vec<Uuid> = (0u128..=100).map(Uuid::from_u128).collect(); // 101 entries
    let filter = DealerFilter {
        dealer_id: Some(ids),
        ..DealerFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(phase_contracts)]
#[test]
fn dealer_filter_limit_over_100_is_invalid() {
    let filter = DealerFilter {
        limit: 101,
        ..DealerFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

// ── Phase 3.6: FacetField wire values ────────────────────────────────────────

#[test]
fn facet_field_wire_values() {
    let cases = [
        (FacetField::Make, "make"),
        (FacetField::Model, "model"),
        (FacetField::InventoryType, "inventory_type"),
        (FacetField::Year, "year"),
        (FacetField::Trim, "trim"),
        (FacetField::Version, "version"),
        (FacetField::BaseExteriorColor, "base_exterior_color"),
        (FacetField::ExteriorColor, "exterior_color"),
        (FacetField::BaseInteriorColor, "base_interior_color"),
        (FacetField::InteriorColor, "interior_color"),
        (FacetField::SeatingCapacity, "seating_capacity"),
        (FacetField::Doors, "doors"),
        (FacetField::Engine, "engine"),
        (FacetField::State, "state"),
        (FacetField::Drivetrain, "drivetrain"),
        (FacetField::AssemblyLocation, "assembly_location"),
        (FacetField::AssemblyCountry, "assembly_country"),
        (FacetField::Transmission, "transmission"),
        (FacetField::FuelType, "fuel_type"),
        (FacetField::BodyType, "body_type"),
        (FacetField::Cylinders, "cylinders"),
        (FacetField::DealerType, "dealer_type"),
        (FacetField::AvailabilityStatus, "availability_status"),
        (FacetField::OptionsPackages, "options_packages"),
        (FacetField::Features, "features"),
        (FacetField::Keywords, "keywords"),
        (FacetField::Price, "price"),
        (FacetField::Msrp, "msrp"),
        (FacetField::Miles, "miles"),
        (FacetField::DaysOnMarket, "days_on_market"),
    ];
    for (field, expected) in cases {
        assert_eq!(
            field.as_str(),
            expected,
            "wrong wire value for {expected:?}"
        );
    }
}

#[test]
fn facet_field_categorical_classification() {
    let categorical = [
        FacetField::Make,
        FacetField::Model,
        FacetField::InventoryType,
        FacetField::Year,
        FacetField::Trim,
        FacetField::Version,
        FacetField::BaseExteriorColor,
        FacetField::ExteriorColor,
        FacetField::BaseInteriorColor,
        FacetField::InteriorColor,
        FacetField::SeatingCapacity,
        FacetField::Doors,
        FacetField::Engine,
        FacetField::State,
        FacetField::Drivetrain,
        FacetField::AssemblyLocation,
        FacetField::AssemblyCountry,
        FacetField::Transmission,
        FacetField::FuelType,
        FacetField::BodyType,
        FacetField::Cylinders,
        FacetField::DealerType,
        FacetField::AvailabilityStatus,
        FacetField::OptionsPackages,
        FacetField::Features,
        FacetField::Keywords,
    ];
    for field in categorical {
        assert!(
            field.is_categorical(),
            "'{}' should be categorical",
            field.as_str()
        );
        assert!(
            !field.is_numeric_range(),
            "'{}' should not be numeric range",
            field.as_str()
        );
    }
}

#[test]
fn facet_field_numeric_range_classification() {
    let numeric = [
        FacetField::Price,
        FacetField::Msrp,
        FacetField::Miles,
        FacetField::DaysOnMarket,
    ];
    for field in numeric {
        assert!(
            field.is_numeric_range(),
            "'{}' should be numeric range",
            field.as_str()
        );
        assert!(
            !field.is_categorical(),
            "'{}' should not be categorical",
            field.as_str()
        );
    }
}

#[test]
fn facet_field_year_is_categorical() {
    assert!(FacetField::Year.is_categorical());
    assert!(!FacetField::Year.is_numeric_range());
}

// ── Phase 3.6: FacetMetricMeasure wire values ─────────────────────────────────

#[test]
fn facet_metric_measure_wire_values() {
    let cases = [
        (FacetMetricMeasure::Price, "price"),
        (FacetMetricMeasure::Miles, "miles"),
        (FacetMetricMeasure::Msrp, "msrp"),
        (FacetMetricMeasure::DaysOnMarket, "days_on_market"),
        (FacetMetricMeasure::DiscountFromMsrp, "discount_from_msrp"),
    ];
    for (measure, expected) in cases {
        assert_eq!(
            measure.as_str(),
            expected,
            "wrong wire value for {expected:?}"
        );
    }
}

// ── Phase 3.6: FacetMetricAggregate wire values ───────────────────────────────

#[test]
fn facet_metric_aggregate_wire_values() {
    let cases = [
        (FacetMetricAggregate::Mean, "mean"),
        (FacetMetricAggregate::P5, "p5"),
        (FacetMetricAggregate::P25, "p25"),
        (FacetMetricAggregate::Median, "median"),
        (FacetMetricAggregate::P75, "p75"),
        (FacetMetricAggregate::P95, "p95"),
    ];
    for (agg, expected) in cases {
        assert_eq!(agg.as_str(), expected, "wrong wire value for {expected:?}");
    }
}

// ── Phase 3.6: FacetMetric serialization ─────────────────────────────────────

#[test]
fn facet_metric_count_serializes_to_count() {
    assert_eq!(FacetMetric::Count.as_str(), "count");
}

#[test]
fn facet_metric_aggregate_price_p95() {
    let metric = FacetMetric::Aggregate {
        measure: FacetMetricMeasure::Price,
        aggregate: FacetMetricAggregate::P95,
    };
    assert_eq!(metric.as_str(), "price.p95");
}

#[test]
fn facet_metric_aggregate_days_on_market_median() {
    let metric = FacetMetric::Aggregate {
        measure: FacetMetricMeasure::DaysOnMarket,
        aggregate: FacetMetricAggregate::Median,
    };
    assert_eq!(metric.as_str(), "days_on_market.median");
}

#[test]
fn facet_metric_aggregate_discount_from_msrp_mean() {
    let metric = FacetMetric::Aggregate {
        measure: FacetMetricMeasure::DiscountFromMsrp,
        aggregate: FacetMetricAggregate::Mean,
    };
    assert_eq!(metric.as_str(), "discount_from_msrp.mean");
}

// ── Phase 3.6: FacetSort wire values ─────────────────────────────────────────

#[test]
fn facet_sort_as_str_wire_values() {
    assert_eq!(FacetSort::Count.as_str(), "count");
    assert_eq!(FacetSort::CountDesc.as_str(), "-count");
    assert_eq!(FacetSort::Metric.as_str(), "metric");
    assert_eq!(FacetSort::MetricDesc.as_str(), "-metric");
}

// ── Phase 3.6: FacetsFilter::new defaults ────────────────────────────────────

#[test]
fn facets_filter_new_default_sort_is_count_desc() {
    let filter = FacetsFilter::new(vec![FacetField::Make]);
    assert!(matches!(filter.sort, FacetSort::CountDesc));
}

#[test]
fn facets_filter_new_metric_is_none() {
    let filter = FacetsFilter::new(vec![FacetField::Make]);
    assert!(filter.metric.is_none());
}

#[test]
fn facets_filter_new_facet_value_limit_is_none() {
    let filter = FacetsFilter::new(vec![FacetField::Make]);
    assert!(filter.facet_value_limit.is_none());
}

// ── Phase 3.6: FacetsFilter validation ───────────────────────────────────────

fn assert_invalid_facet_filter(result: Result<(), VisorError>) {
    assert!(
        matches!(result, Err(VisorError::InvalidFilter { .. })),
        "expected InvalidFilter, got: {result:?}"
    );
}

#[test]
fn facets_filter_valid_single_categorical_facet() {
    assert!(FacetsFilter::new(vec![FacetField::Make]).validate().is_ok());
}

#[test]
fn facets_filter_valid_multiple_categorical_facets_with_count_metric() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make, FacetField::Model]);
    filter.metric = Some(FacetMetric::Count);
    assert!(filter.validate().is_ok());
}

#[test]
fn facets_filter_empty_facets_rejected() {
    assert_invalid_facet_filter(FacetsFilter::new(vec![]).validate());
}

#[test]
fn facets_filter_facet_value_limit_100_is_valid() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make]);
    filter.facet_value_limit = Some(100);
    assert!(filter.validate().is_ok());
}

#[test]
fn facets_filter_facet_value_limit_over_100_rejected() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make]);
    filter.facet_value_limit = Some(101);
    assert_invalid_facet_filter(filter.validate());
}

#[test]
fn facets_filter_aggregate_metric_requires_exactly_one_facet_zero_rejected() {
    // Zero facets is already caught by the empty-facets check; validate ordering here.
    let mut filter = FacetsFilter::new(vec![]);
    filter.metric = Some(FacetMetric::Aggregate {
        measure: FacetMetricMeasure::Price,
        aggregate: FacetMetricAggregate::P95,
    });
    assert_invalid_facet_filter(filter.validate());
}

#[test]
fn facets_filter_aggregate_metric_with_multiple_facets_rejected() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make, FacetField::Model]);
    filter.metric = Some(FacetMetric::Aggregate {
        measure: FacetMetricMeasure::Price,
        aggregate: FacetMetricAggregate::Mean,
    });
    assert_invalid_facet_filter(filter.validate());
}

#[test]
fn facets_filter_aggregate_metric_with_numeric_range_facet_rejected() {
    let mut filter = FacetsFilter::new(vec![FacetField::Price]);
    filter.metric = Some(FacetMetric::Aggregate {
        measure: FacetMetricMeasure::Price,
        aggregate: FacetMetricAggregate::Median,
    });
    assert_invalid_facet_filter(filter.validate());
}

#[test]
fn facets_filter_aggregate_metric_with_categorical_facet_valid() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make]);
    filter.metric = Some(FacetMetric::Aggregate {
        measure: FacetMetricMeasure::Price,
        aggregate: FacetMetricAggregate::P95,
    });
    assert!(filter.validate().is_ok());
}

#[test]
fn facets_filter_metric_sort_without_aggregate_metric_rejected() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make]);
    filter.sort = FacetSort::Metric;
    assert_invalid_facet_filter(filter.validate());
}

#[test]
fn facets_filter_metric_desc_sort_without_aggregate_metric_rejected() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make]);
    filter.sort = FacetSort::MetricDesc;
    assert_invalid_facet_filter(filter.validate());
}

#[test]
fn facets_filter_metric_sort_with_count_metric_rejected() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make]);
    filter.metric = Some(FacetMetric::Count);
    filter.sort = FacetSort::Metric;
    assert_invalid_facet_filter(filter.validate());
}

#[test]
fn facets_filter_metric_sort_with_aggregate_metric_valid() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make]);
    filter.metric = Some(FacetMetric::Aggregate {
        measure: FacetMetricMeasure::Price,
        aggregate: FacetMetricAggregate::P95,
    });
    filter.sort = FacetSort::MetricDesc;
    assert!(filter.validate().is_ok());
}

#[test]
fn facets_filter_count_sort_with_aggregate_metric_valid() {
    let mut filter = FacetsFilter::new(vec![FacetField::Make]);
    filter.metric = Some(FacetMetric::Aggregate {
        measure: FacetMetricMeasure::Miles,
        aggregate: FacetMetricAggregate::Mean,
    });
    filter.sort = FacetSort::Count;
    assert!(filter.validate().is_ok());
}
