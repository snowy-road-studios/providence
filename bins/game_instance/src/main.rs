use bevy_girk_game_instance::*;
use clap::Parser;
use wiring_game_instance::*;

#[allow(unused_imports)]
#[macro_use]
extern crate static_assertions;

//-------------------------------------------------------------------------------------------------------------------

fn main()
{
    #[cfg(not(feature = "commands"))]
    {
        const_assert!(!game_core::CommandInput::command_processing_enabled());
    }

    // log to stderr (not stdout, which is piped to the parent process for sending game instance reports)
    //todo: log to file instead (use env::arg configs?)
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env()
        .unwrap();
    #[cfg(feature = "dev")]
    let filter = filter
        .add_directive("renet2=info".parse().unwrap())
        .add_directive("renet2_netcode=info".parse().unwrap())
        .add_directive("renetcode2=info".parse().unwrap())
        .add_directive("bevy_girk_host_server=trace".parse().unwrap())
        .add_directive("bevy_girk_game_hub_server=trace".parse().unwrap())
        .add_directive("bevy_girk_wiring=trace".parse().unwrap())
        .add_directive("game_core=trace".parse().unwrap())
        .add_directive("bevy_girk_game_fw=trace".parse().unwrap());
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    // make game factory
    let game_factory = GameFactory::new(ProvGameFactory {});

    // launch the game
    let args = GameInstanceCli::parse();
    inprocess_game_launcher(args, game_factory);
}

//-------------------------------------------------------------------------------------------------------------------
