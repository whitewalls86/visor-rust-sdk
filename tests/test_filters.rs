use visor::{
    DealerFilter, DealerType, FacetSort, FacetsFilter, InventoryStatus, ListingInclude,
    ListingsFilter, ListingsFilterBase, SortOrder, VisorError,
};

fn param(params: &[(String, String)], key: &str) -> Option<String> {
    params
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
}

fn has_key(params: &[(String, String)], key: &str) -> bool {
    params.iter().any(|(k, _)| k == key)
}

// ── Serialization golden tests ────────────────────────────────────────────────

#[cfg(feature = "phase-contracts")]
#[test]
fn default_listings_filter_emits_limit_offset_sort() {
    let params = ListingsFilter::default().to_params();
    assert_eq!(param(&params, "limit").as_deref(), Some("50"));
    assert_eq!(param(&params, "offset").as_deref(), Some("0"));
    assert_eq!(param(&params, "sort").as_deref(), Some("days_on_market"));
}

#[test]
fn inventory_status_active_omitted_from_params() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            inventory_status: InventoryStatus::Active,
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert!(!has_key(&filter.to_params(), "inventory_status"));
}

#[cfg(feature = "phase-contracts")]
#[test]
fn inventory_status_sold_emitted_as_wire_value() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            inventory_status: InventoryStatus::Sold,
            sold_within_days: Some(30),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    let params = filter.to_params();
    assert_eq!(param(&params, "inventory_status").as_deref(), Some("sold"));
    assert_eq!(param(&params, "sold_within_days").as_deref(), Some("30"));
}

#[cfg(feature = "phase-contracts")]
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

#[cfg(feature = "phase-contracts")]
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

#[cfg(feature = "phase-contracts")]
#[test]
fn bbox_serialized_as_west_south_east_north() {
    use visor::BBox;
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            bbox: Some(BBox {
                west: -122.5,
                south: 37.2,
                east: -121.9,
                north: 37.8,
            }),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "bbox").as_deref(),
        Some("-122.5,37.2,-121.9,37.8")
    );
}

#[cfg(feature = "phase-contracts")]
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

#[cfg(feature = "phase-contracts")]
#[test]
fn fields_projection_emitted_comma_separated() {
    let filter = ListingsFilter {
        fields: Some(vec![
            "make".to_string(),
            "price".to_string(),
            "year".to_string(),
        ]),
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "fields").as_deref(),
        Some("make,price,year")
    );
}

#[cfg(feature = "phase-contracts")]
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

#[cfg(feature = "phase-contracts")]
#[test]
fn snapshot_date_serialized_as_iso8601() {
    use chrono::NaiveDate;
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            snapshot_date: Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_eq!(
        param(&filter.to_params(), "snapshot_date").as_deref(),
        Some("2024-03-15")
    );
}

#[cfg(feature = "phase-contracts")]
#[test]
fn comma_separated_list_fields_join_correctly() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            make: Some(vec!["Toyota".to_string(), "Honda".to_string()]),
            state: Some(vec!["CA".to_string(), "TX".to_string()]),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    let params = filter.to_params();
    assert_eq!(param(&params, "make").as_deref(), Some("Toyota,Honda"));
    assert_eq!(param(&params, "state").as_deref(), Some("CA,TX"));
}

#[cfg(feature = "phase-contracts")]
#[test]
fn facets_filter_always_emits_sort() {
    let filter = FacetsFilter::new(vec!["make".to_string()]);
    let params = filter.to_params();
    assert!(
        has_key(&params, "sort"),
        "FacetsFilter must always emit sort"
    );
    // CountDesc is the default
    assert_eq!(param(&params, "sort").as_deref(), Some("-count"));
}

#[cfg(feature = "phase-contracts")]
#[test]
fn facet_sort_wire_values() {
    let cases = [
        (FacetSort::Count, "count"),
        (FacetSort::CountDesc, "-count"),
        (FacetSort::Metric, "metric"),
        (FacetSort::MetricDesc, "-metric"),
    ];
    for (sort, expected) in cases {
        let mut filter = FacetsFilter::new(vec!["make".to_string()]);
        filter.sort = sort;
        assert_eq!(
            param(&filter.to_params(), "sort").as_deref(),
            Some(expected),
            "wrong wire value for FacetSort variant"
        );
    }
}

#[cfg(feature = "phase-contracts")]
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

#[cfg(feature = "phase-contracts")]
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

#[cfg(feature = "phase-contracts")]
#[test]
fn dealer_filter_default_emits_limit_and_offset() {
    let params = DealerFilter::default().to_params();
    assert_eq!(param(&params, "limit").as_deref(), Some("50"));
    assert_eq!(param(&params, "offset").as_deref(), Some("0"));
}

// ── Validation error tests ────────────────────────────────────────────────────

fn assert_invalid_filter(result: Result<(), VisorError>) {
    assert!(
        matches!(result, Err(VisorError::InvalidFilter { .. })),
        "expected InvalidFilter, got: {result:?}"
    );
}

#[cfg(feature = "phase-contracts")]
#[test]
fn radius_without_any_anchor_is_invalid() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            radius: Some(25.0),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(feature = "phase-contracts")]
#[test]
fn radius_with_both_anchors_is_invalid() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            radius: Some(25.0),
            postal_code: Some("90210".to_string()),
            latitude: Some(34.0),
            longitude: Some(-118.0),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(feature = "phase-contracts")]
#[test]
fn radius_with_lat_but_no_lon_is_invalid() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            radius: Some(25.0),
            latitude: Some(34.0),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(feature = "phase-contracts")]
#[test]
fn bbox_and_radius_together_is_invalid() {
    use visor::BBox;
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            bbox: Some(BBox {
                west: -122.5,
                south: 37.2,
                east: -121.9,
                north: 37.8,
            }),
            radius: Some(10.0),
            postal_code: Some("94102".to_string()),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(feature = "phase-contracts")]
#[test]
fn sold_within_days_without_sold_status_is_invalid() {
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            sold_within_days: Some(30),
            // inventory_status defaults to Active
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(feature = "phase-contracts")]
#[test]
fn snapshot_date_with_sold_status_is_invalid() {
    use chrono::NaiveDate;
    // snapshot_date requires Active; Sold is not Active
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            snapshot_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            inventory_status: InventoryStatus::Sold,
            sold_within_days: Some(30),
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(feature = "phase-contracts")]
#[test]
fn sold_within_days_and_snapshot_date_together_is_invalid() {
    use chrono::NaiveDate;
    // sold_within_days needs Sold; snapshot_date needs Active — mutually exclusive.
    // Setting inventory_status=Active will fail on sold_within_days needing Sold.
    // Either way, validate() must return InvalidFilter.
    let filter = ListingsFilter {
        base: ListingsFilterBase {
            sold_within_days: Some(30),
            snapshot_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            inventory_status: InventoryStatus::Active,
            ..ListingsFilterBase::default()
        },
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(feature = "phase-contracts")]
#[test]
fn listings_limit_over_100_is_invalid() {
    let filter = ListingsFilter {
        limit: 101,
        ..ListingsFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(feature = "phase-contracts")]
#[test]
fn facets_filter_empty_facets_is_invalid() {
    assert_invalid_filter(FacetsFilter::new(vec![]).validate());
}

#[cfg(feature = "phase-contracts")]
#[test]
fn dealer_filter_over_100_ids_is_invalid() {
    let ids: Vec<String> = (0..=100).map(|i| format!("dealer-{i}")).collect(); // 101 entries
    let filter = DealerFilter {
        dealer_id: Some(ids),
        ..DealerFilter::default()
    };
    assert_invalid_filter(filter.validate());
}

#[cfg(feature = "phase-contracts")]
#[test]
fn dealer_filter_limit_over_100_is_invalid() {
    let filter = DealerFilter {
        limit: 101,
        ..DealerFilter::default()
    };
    assert_invalid_filter(filter.validate());
}
