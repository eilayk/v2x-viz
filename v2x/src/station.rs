use c_its_parser::standards::cdd_1_3_1_1::its_container::StationType;

/// ETSI ITS station type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ItsStationType {
    /// Passenger car.
    PassengerCar = 5,
}

impl From<ItsStationType> for StationType {
    fn from(st: ItsStationType) -> StationType {
        StationType(st as u8)
    }
}
