use bevy_girk_utils::Rand64;

//-------------------------------------------------------------------------------------------------------------------

pub fn protocol_id() -> u64
{
    let domain_sep = concat!("providence game protocol id: v", env!("CARGO_PKG_VERSION"));
    let protocol_id = Rand64::new(domain_sep, 0u128).next();
    protocol_id
}

//-------------------------------------------------------------------------------------------------------------------
