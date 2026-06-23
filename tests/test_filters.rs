use visor::{InventoryModeFilter, ListingsFilter, ListingsFilterBase};

// Phase 3.6 types — always available, used in unconditional tests.
use visor::{
    FacetField, FacetMetric, FacetMetricAggregate, FacetMetricMeasure, FacetSort, FacetsFilter,
    VisorError,
};

#[cfg(phase_contracts)]
use uuid::Uuid;

#[cfg(phase_contracts)]
use visor::{
    BBox, DealerFilter, DealerType, GeoFilter, HistoryKeyword, InventoryType, ListingField,
    ListingInclude, SortOrder, StateCode,
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
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            geo: Some(GeoFilter::BBox(BBox {
                west: -122.5,
                south: 37.2,
                east: -121.9,
                north: 37.8,
            })),
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
fn facets_filter_empty_facets_is_invalid() {
    assert_invalid_filter(FacetsFilter::new(vec![]).validate());
}

#[cfg(phase_contracts)]
#[test]
fn dealer_filter_over_50_ids_is_invalid() {
    // dealer_id takes Vec<Uuid>; generate 51 deterministic nil-variant UUIDs.
    let ids: Vec<Uuid> = (0u128..=50).map(Uuid::from_u128).collect(); // 51 entries
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
