use std::collections::HashMap;
use once_cell::sync::Lazy;

pub(crate) struct CovidRegion {
    pub(crate) region: &'static str,
    pub(crate) region_type: &'static str
}

pub(crate) static UK_CARBON_REGIONS: Lazy<HashMap<u32, &str>> = Lazy::new(||
    HashMap::from([
        (1, "North Scotland"),
        (2, "South Scotland"),
        (3, "North West England"),
        (4, "North East England"),
        (5, "Yorkshire"),
        (6, "North Wales"),
        (7, "South Wales"),
        (8, "West Midlands"),
        (9, "East Midlands"),
        (10, "East England"),
        (11, "South West England"),
        (12, "South England"),
        (13, "London"),
        (14, "South East England"),
        (15, "England"),
        (16, "Scotland"),
        (17, "Wales"),
    ])
);

pub(crate) static UK_CARBON_TO_COVID_REGIONS: Lazy<HashMap<u32, CovidRegion>> = Lazy::new(||
    HashMap::from([
        (1, CovidRegion{region: "Scotland", region_type: "nation" }),
        (2, CovidRegion{region: "Scotland", region_type: "nation" }),
        (3, CovidRegion{region: "North West", region_type: "region" }),
        (4, CovidRegion{region: "North East", region_type: "region" }),
        (5, CovidRegion{region: "Yorkshire and The Humber", region_type: "region" }),
        (6, CovidRegion{region: "Wales", region_type: "nation" }),
        (7, CovidRegion{region: "Wales", region_type: "nation" }),
        (8, CovidRegion{region: "West Midlands", region_type: "region" }),
        (9, CovidRegion{region: "East MidLands", region_type: "region" }),
        (10, CovidRegion{region: "East of England", region_type: "region" }),
        (11, CovidRegion{region: "South West", region_type: "region" }),
        (12, CovidRegion{region: "England", region_type: "nation" }),
        (13, CovidRegion{region: "London", region_type: "region" }),
        (14, CovidRegion{region: "South East", region_type: "region" }),
        (15, CovidRegion{region: "England", region_type: "nation" }),
        (16, CovidRegion{region: "Scotland", region_type: "nation" }),
        (17, CovidRegion{region: "Wales", region_type: "nation" }),
    ])
);