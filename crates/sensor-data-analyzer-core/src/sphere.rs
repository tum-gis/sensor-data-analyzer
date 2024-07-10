use ecoord::UnitSphericalPoint3;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct UnitSphericalCellIndex3 {
    azimuth: i32,
    elevation: i32,
}

impl UnitSphericalCellIndex3 {
    pub fn new(azimuth: i32, elevation: i32) -> Self {
        Self { azimuth, elevation }
    }

    pub fn azimuth(&self) -> i32 {
        self.azimuth
    }

    pub fn elevation(&self) -> i32 {
        self.elevation
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphericalRasterizationTransform {
    azimuth: SphericalRasterizationAxis,
    elevation: SphericalRasterizationAxis,
}

impl SphericalRasterizationTransform {
    pub fn new(azimuth: SphericalRasterizationAxis, elevation: SphericalRasterizationAxis) -> Self {
        Self { azimuth, elevation }
    }

    pub fn azimuth(&self) -> SphericalRasterizationAxis {
        self.azimuth
    }

    pub fn elevation(&self) -> SphericalRasterizationAxis {
        self.elevation
    }

    pub fn transform_to_point(
        &self,
        cell_index: UnitSphericalCellIndex3,
    ) -> UnitSphericalPoint3<f64> {
        let azimuth = self
            .azimuth
            .transform_to_continues_value_rad(cell_index.azimuth);
        let elevation = self
            .elevation
            .transform_to_continues_value_rad(cell_index.elevation);

        UnitSphericalPoint3::new(azimuth, elevation)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphericalRasterizationAxis {
    field_of_view_min: f64,
    field_of_view_max: f64,
    resolution: f64,
    offset: f64,
}

impl SphericalRasterizationAxis {
    pub fn new(
        field_of_view_min: f64,
        field_of_view_max: f64,
        resolution: f64,
        offset: f64,
    ) -> Self {
        Self {
            field_of_view_min,
            field_of_view_max,
            resolution,
            offset,
        }
    }

    pub fn from_deg(
        field_of_view_min_deg: f64,
        field_of_view_max_deg: f64,
        resolution_deg: f64,
        offset_deg: f64,
    ) -> Self {
        Self {
            field_of_view_min: field_of_view_min_deg.to_radians(),
            field_of_view_max: field_of_view_max_deg.to_radians(),
            resolution: resolution_deg.to_radians(),
            offset: offset_deg.to_radians(),
        }
    }

    pub fn offset(&self) -> f64 {
        self.offset
    }

    pub fn range_deg(&self) -> f64 {
        self.range().to_degrees()
    }

    pub fn range(&self) -> f64 {
        self.field_of_view_max - self.field_of_view_min
    }

    pub fn resolution(&self) -> f64 {
        self.resolution
    }

    /*pub fn grid_size(&self) -> f64 {
        self.range_deg() / self.resolution_deg
    }*/

    pub fn transform_to_grid_cell_index(&self, value: f64) -> i32 {
        ((value + self.offset) / self.resolution) as i32
    }

    pub fn transform_to_continues_value_rad(&self, index: i32) -> f64 {
        index as f64 * self.resolution - self.offset
    }
}
