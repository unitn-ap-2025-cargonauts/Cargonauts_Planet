//! # Cargonauts Planet AI Module
//!
//! This module contains the implementation of the `PlanetAI` trait for the
//! Cargonauts planet. It defines how the planet handle messages from the
//! Orchestrator and the Explorer.
//!
//! Each handler is defined as a standalone function to keep the logic modular and clean.

use crossbeam_channel::{Sender, Receiver};
use common_game::components::planet::*;
use common_game::components::resource::*;
use common_game::components::rocket::Rocket;
use common_game::logging::{ActorType, Channel, EventType, LogEvent, Payload};
use common_game::protocols::messages::*;
use paste::paste;


#[allow(dead_code)]
trait PlanetDefinition {
    fn get_name() -> &'static str;
    fn get_type() -> &'static PlanetType;
}


struct CargonautsPlanet;



/// Function that create a Planet with specific arguments TODO
#[allow(dead_code)]
pub fn create_planet(
    rx_orchestrator: Receiver<OrchestratorToPlanet>,
    tx_orchestrator: Sender<PlanetToOrchestrator>,
    rx_explorer: Receiver<ExplorerToPlanet>,
    planet_id : u32,
) -> Planet {
    let planet = Planet::new(
        planet_id,
        PlanetType::C,
        Box::new(CargonautsPlanet::default()),
        vec![BasicResourceType::Carbon],
        vec![
            ComplexResourceType::Diamond,
            ComplexResourceType::Life,
            ComplexResourceType::AIPartner,
            ComplexResourceType::Dolphin,
            ComplexResourceType::Robot,
            ComplexResourceType::Water
        ],
        (rx_orchestrator, tx_orchestrator),
        rx_explorer
    );
    assert!(planet.is_ok(), "Planet creation error!");
    planet.unwrap()
}

impl PlanetDefinition for CargonautsPlanet {
    fn get_name() -> &'static str {
        "Cargonauts Planet"
    }

    fn get_type() -> &'static PlanetType {
        &PlanetType::C
    }
}


impl Default for CargonautsPlanet {
    fn default() -> Self {
        Self
    }
}

impl PlanetAI for CargonautsPlanet {

    fn handle_orchestrator_msg(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        msg: OrchestratorToPlanet,
    ) -> Option<PlanetToOrchestrator> {
        match msg {
            OrchestratorToPlanet::Sunray(sunray) => {
                let _ = state.charge_cell(sunray);
                Some(PlanetToOrchestrator::SunrayAck {
                    planet_id: state.id(),
                })
            }

            OrchestratorToPlanet::InternalStateRequest => {
                Some(PlanetToOrchestrator::InternalStateResponse {
                    planet_id: state.id(),
                    planet_state: state.to_dummy(),
                })
            }

            /* All the other messages (Start,Stop,Asteroid,Explorer...)
             are already handled by the 'run' loop on planet.rs
             If, for some reason, they get here, we ignore it
             */
            _ => None,
        }
    }


    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet
    ) -> Option<PlanetToExplorer> {
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id } => {
                let mut payload = Payload::new();
                payload.insert("msg_type".to_string(), "SupportedResourceRequest".to_string());
                LogEvent::new(
                    ActorType::Explorer,
                    explorer_id, ActorType::Planet,
                    state.id().to_string(),
                    EventType::MessageExplorerToPlanet,
                    Channel::Info,
                    payload
                ).emit();

                handle_supported_resource_request(generator)
            },
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                let mut payload = Payload::new();
                payload.insert("msg_type".to_string(), "SupportedCombinationRequest".to_string());
                LogEvent::new(
                    ActorType::Explorer,
                    explorer_id, ActorType::Planet,
                    state.id().to_string(),
                    EventType::MessageExplorerToPlanet,
                    Channel::Info,
                    payload
                ).emit();

                handle_supported_combination_request(combinator)
            },
            ExplorerToPlanet::GenerateResourceRequest { explorer_id, resource } => {
                let mut payload = Payload::new();
                payload.insert("msg_type".to_string(), "GenerateResourceRequest".to_string());
                LogEvent::new(
                    ActorType::Explorer,
                    explorer_id, ActorType::Planet,
                    state.id().to_string(),
                    EventType::MessageExplorerToPlanet,
                    Channel::Info,
                    payload
                ).emit();

                handle_generate_resource_request(state, generator, resource)
            },
            ExplorerToPlanet::CombineResourceRequest { explorer_id, msg } => {
                let mut payload = Payload::new();
                payload.insert("msg_type".to_string(), "CombineResourceRequest".to_string());
                LogEvent::new(
                    ActorType::Explorer,
                    explorer_id, ActorType::Planet,
                    state.id().to_string(),
                    EventType::MessageExplorerToPlanet,
                    Channel::Info,
                    payload
                ).emit();

                handle_combine_resource_request(state, combinator, msg)
            },
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id } => {
                let mut payload = Payload::new();
                payload.insert("msg_type".to_string(), "AvailableEnergyCellRequest".to_string());
                LogEvent::new(
                    ActorType::Explorer,
                    explorer_id, ActorType::Planet,
                    state.id().to_string(),
                    EventType::MessageExplorerToPlanet,
                    Channel::Info,
                    payload
                ).emit();

                handle_energy_cell_request(state)
            },
        }
    }


    /// Handler for the [Asteroid] message, it returns `None` or `Some([Rocket])` based on the rules of the
    /// [Planet] or the availability of [Rocket] on the planet.
    ///
    /// More precisely, it returns `None` if:
    /// - The [Planet] can not create any [Rocket] because of its rules.
    /// - The [Planet] can not crate any [Rocket] because it has no charged [EnergyCell].
    ///
    /// It returns `Some(Rocket)` if:
    /// - The [Planet]'s rules allow it to do so and it already has a [Rocket] that can be used.
    /// - The [Planet]'s rules allow it to do so and it was able to build a [Rocket] when [Asteroid]
    /// message was delivered to it.
    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _: &Generator,
        _: &Combinator
    ) -> Option<Rocket> {


        // At this point the Rocket can be built. Check if there already
        // is a rocket ready to be used
        if state.has_rocket() {


            let before_take_rocket = format!("state.has_rocket() before .take_rocket(..) : {{ state.has_rocket() {} }}", state.has_rocket());


            let rocket = state.take_rocket();

            if let Some(taken_rocket) = rocket {
                logging_wrapper::log_for_channel_info(
                    state.id(),
                    ActorType::SelfActor,
                    0.to_string(),
                    EventType::InternalPlanetAction,
                    vec!["Planet received an Asteroid and has the rocket to deflect it. Proceeding".to_string()]
                );


                logging_wrapper::log_for_channel_debug(
                    state.id(),
                    ActorType::SelfActor,
                    0.to_string(),
                    EventType::InternalPlanetAction,
                    vec![ before_take_rocket, "state.rocket: None".to_string()  ]
                );

                Some(taken_rocket)
            } else {

                logging_wrapper::log_for_channel_with_key_error(
                    state.id(),
                    ActorType::SelfActor,
                    0.to_string(),
                    EventType::InternalPlanetAction,
                    logging_wrapper::drop_planet_state_fields_as_vector(&state)
                );

                None
            }
        } else {

            //The Rocket is not available, try to build it
            let charged_cell_position = state.cells_iter().position( |cell_ref| cell_ref.is_charged() );
            if let Some(charged_cell_position_result) = charged_cell_position {
                // There is a charged cell, therefore a Rocket can be built


                // Build the rocket
                let created_rocket_result = state.build_rocket( charged_cell_position_result );

                // Check the Result
                match created_rocket_result {
                    Ok(_) => {
                        // Successfully created the rocket

                        logging_wrapper::log_for_channel_info(
                            state.id(),
                            ActorType::SelfActor,
                            0.to_string(),
                            EventType::InternalPlanetAction,
                            vec!["Planet received an Asteroid and has just created a rocket to deflect it. Proceeding".to_string()]
                        );


                        logging_wrapper::log_for_channel_with_key_debug(
                            state.id(),
                            ActorType::SelfActor,
                            0.to_string(),
                            EventType::InternalPlanetAction,
                            logging_wrapper::drop_planet_state_fields_as_vector( &state )
                        );

                        // Give ownership to the orchestrator
                        state.take_rocket()
                    },
                    Err(string_error) => {
                        // Technically the rocket could be built but something went wrong; logging the error
                        logging_wrapper::log_for_channel_error(state.id(), ActorType::SelfActor, 0.to_string(), EventType::InternalPlanetAction, vec!["The build_rocket() failed!".to_string()]);
                        let mut state_vec = vec![("Event type".to_string(), "build_rocket() failed".to_string()), ( "Received error message".to_string() , format!("{}", string_error).to_string() )];
                        logging_wrapper::drop_planet_state_fields_as_vector(&state).iter().for_each(|(key, value)|  state_vec.push( (key.to_string(), value.to_string()) ) );
                        logging_wrapper::log_for_channel_with_key_trace(state.id(), ActorType::SelfActor, 0.to_string(), EventType::InternalPlanetAction, state_vec);
                        None
                    }
                }
            } else {
                // Rocket can not be built
                logging_wrapper::log_for_channel_info(state.id(), ActorType::SelfActor, 0.to_string(), EventType::InternalPlanetAction, vec!["Planet received an Asteroid and does not have any rocket to deflect it.".to_string()]);
                logging_wrapper::log_for_channel_with_key_debug(state.id(), ActorType::SelfActor,0.to_string(),EventType::InternalPlanetAction,  logging_wrapper::drop_planet_state_fields_as_vector(&state) );
                None
            }
        }
    }


    /// This method will be invoked when a [OrchestratorToPlanet::StartPlanetAI]
    /// is received, but **only if** the planet is currently in a *stopped* state.
    ///
    /// Start messages received when planet is already running are **ignored**.
    fn start(&mut self, state: &PlanetState) {
        logging_wrapper::log_for_channel_with_key_debug(
            state.id(),
            ActorType::SelfActor,
            0.to_string(),
            EventType::InternalPlanetAction,
            logging_wrapper::drop_planet_state_fields_as_vector(&state)
            //vec!["Planet is starting its AI".to_string()]
        );
    }


    /// This method will be invoked when a [OrchestratorToPlanet::StopPlanetAI]
    /// is received, but **only if** the planet is currently in a *running* state.
    ///
    fn stop(&mut self, state: &PlanetState) {
        logging_wrapper::log_for_channel_warning(state.id(), ActorType::SelfActor, 0.to_string(), EventType::InternalPlanetAction,  vec!["Planet is being disabled. All messages will be ignored (except for KillPlanet, StartPlanetAI) ".to_string()]);
    }
}

// === ExplorerToPlanet Handler ====================================================================
/// This handler returns a [SupportedResourceResponse] message that wrap the list of basic resources
/// that the planet can currently generate
///
/// # Parameters
/// - `generator`: Reference to the planet's generator
///
/// # Returns
/// `Some(PlanetToExplorer::SupportedResourceResponse)`
///
/// # Panics
/// This function does not panic.
///
/// # Logic
/// The planet can craft basic resources, so the handler:
/// - Get the set of available basic resource from the planet generator
/// - Wrap the set in a [SupportedResourceResponse] message and return it
#[allow(dead_code)]
fn handle_supported_resource_request(
    generator: &Generator,
) -> Option<PlanetToExplorer> {
    let resource_list = generator.all_available_recipes();

    Some(PlanetToExplorer::SupportedResourceResponse { resource_list })
}

/// This handler returns a [SupportedCombinationResponse] message that wrap the list of complex resources
/// that the planet can currently generate
///
/// # Parameters
/// - `combinator`: Reference to the planet's combinator
///
/// # Returns
/// `Some(PlanetToExplorer::SupportedCombinationResponse)`
///
/// # Panics
/// This function does not panic.
///
/// # Logic
/// The planet can craft complex resources, so the handler:
/// - Get the set of available complex resource from the planet combinator
/// - Wrap the set in a [SupportedCombinationResponse] message and return it
#[allow(dead_code)]
fn handle_supported_combination_request(
    combinator: &Combinator,
) -> Option<PlanetToExplorer> {
    let combination_list = combinator.all_available_recipes();

    Some(PlanetToExplorer::SupportedCombinationResponse  { combination_list })
}

/// This handler processes a request to generate a basic resource using the planet's generator,
/// if energy is available.
/// It returns a [GenerateResourceResponse] message containing the generated resource.
///
/// # Parameters
/// - `state`: Mutable reference to the planet state.
/// - `generator`: Reference to the planet's generator.
/// - `req_resource`: The type of basic resource the explorer is requesting to generate.
///
/// # Returns
/// `Some(PlanetToExplorer::GenerateResourceResponse)` containing:
/// - `Some(BasicResource::Carbon)` on successful generation.
/// - `None` if the planet has no charged energy cell.
///
/// # Panics
/// - If the requested resource type is not supported by the generator.
/// - If the generator reports an unexpected error while crafting.
///
/// # Logic
/// - Retrieve the energy cell and check if it is charged
/// - If charged:
///     - Attempt to generate the requested basic resource via the generator
///     - Wrap the generated resource in a [GenerateResourceResponse] message and return it.
/// - Else:
///     - Wrap a `None` in a [GenerateResourceResponse] message and return it.
#[allow(dead_code)]
fn handle_generate_resource_request(
    state: &mut PlanetState,
    generator: &Generator,
    req_resource: BasicResourceType,
) -> Option<PlanetToExplorer> {
    let mut resource: Option<BasicResource> = None;
    let energy_cell = state.cell_mut(0);
    if energy_cell.is_charged(){
        match req_resource {
            BasicResourceType::Carbon => {
                match generator.make_carbon(energy_cell){
                    Ok( r) => resource = Some(BasicResource::Carbon(r)),
                    Err(e) => panic!("{:?}", e) //TODO log it
                }
            },
            _ => panic!("Unexpected resource type") //TODO log it
        }
    }
    Some(PlanetToExplorer::GenerateResourceResponse { resource })
}

macro_rules! generate_complex_resource {
    ($combinator:expr, $cell:expr, $msg:expr, { $( $complex_resource:ident ( $r1:ident, $r2:ident ) ),* $(,)? }) => {{
        paste! {
            match $msg {
                $(
                    ComplexResourceRequest::$complex_resource($r1, $r2) => {
                        $combinator
                            .[<make_ $complex_resource:lower>]($r1, $r2, $cell)
                            .map(ComplexResource::$complex_resource)
                            .map_err(|(msg, r1, r2)| (msg, r1.to_generic(), r2.to_generic()))
                    }
                )*
            }
        }
    }};
}

/// This handler processes a request to combine two basic or complex resources
/// into a new complex resource using the planet's combinator, if energy is available.
/// It returns a [CombineResourceResponse] message containing the newly
/// crafted complex resource.
///
/// # Parameters
/// - `state`: Mutable reference to the planet state.
/// - `combinator`: Reference to the planet's combinator.
/// - `msg`: A [ComplexResourceRequest] specifying which complex resource the
///   explorer want to craft, with the ingredients required.
///
/// # Returns
/// `Some(PlanetToExplorer::CombineResourceResponse)` containing:
/// - `Some(ComplexResource::X)` if the combination succeeds.
/// - `None` if the planet has no charged energy cell.
///
/// # Panics
/// - If the requested complex resource type is not supported by the combinator.
/// - If the combinator reports an unexpected error while crafting.
///
/// # Logic
/// - Retrieve the energy cell and check if it is charged.
/// - If charged:
///     - Attempt to combine the provided ingredients using the combinator.
///     - Wrap the produced resource in a [CombineResourceResponse] message and return it.
/// - Else:
///     - Wrap a `None` in a [CombineResourceResponse] message and return it.
#[allow(dead_code)]
fn handle_combine_resource_request(
    state: &mut PlanetState,
    combinator: &Combinator,
    msg: ComplexResourceRequest,
) -> Option<PlanetToExplorer> {
    let energy_cell = state.cell_mut(0);

    if !energy_cell.is_charged() {
        return None;
    }

    /*let complex_response = match msg {
        ComplexResourceRequest::Diamond(c1, c2) => {
            combinator
                .make_diamond(c1, c2, energy_cell)
                .map(ComplexResource::Diamond)
                .map_err(|(msg, r1, r2)| (msg, r1.to_generic(), r2.to_generic()))
        }
        ComplexResourceRequest::Life(w, c) => {
            combinator
                .make_life(w, c, energy_cell)
                .map(ComplexResource::Life)
                .map_err(|(msg, r1, r2)| (msg, r1.to_generic(), r2.to_generic()))
        }
        ComplexResourceRequest::Water(h, o) => {
            combinator
                .make_water(h, o, energy_cell)
                .map(ComplexResource::Water)
                .map_err(|(msg, r1, r2)| (msg, r1.to_generic(), r2.to_generic()))
        }
        ComplexResourceRequest::Robot(s, l) => {
            combinator
                .make_robot(s, l, energy_cell)
                .map(ComplexResource::Robot)
                .map_err(|(msg, r1, r2)| (msg, r1.to_generic(), r2.to_generic()))
        }
        ComplexResourceRequest::Dolphin(w, l) => {
            combinator
                .make_dolphin(w, l, energy_cell)
                .map(ComplexResource::Dolphin)
                .map_err(|(msg, r1, r2)| (msg, r1.to_generic(), r2.to_generic()))
        }
        ComplexResourceRequest::AIPartner(r, d) => {
            combinator
                .make_aipartner(r, d, energy_cell)
                .map(ComplexResource::AIPartner)
                .map_err(|(msg, r1, r2)| (msg, r1.to_generic(), r2.to_generic()))
        }
    };*/

    let complex_response = generate_complex_resource!(combinator, energy_cell, msg, {
        Diamond(c1, c2),
        Life(w, c),
        Water(h, o),
        Robot(s, l),
        Dolphin(w, l),
        AIPartner(r, d),
    });

    Some(PlanetToExplorer::CombineResourceResponse {complex_response})
}

/// This handler returns an [AvailableEnergyCellResponse] message containing
/// the number of currently charged energy cells available on the planet.
/// Since the planet has only one energy cell, the value can only be 0 or 1.
///
/// # Parameters
/// - `state`: Reference to the planet state
///
/// # Returns
/// `Some(PlanetToExplorer::AvailableEnergyCellResponse)` with `available_cells` set to:
/// - **0**: if the energy cell is discharged
/// - **1**: if the energy cell is charged
///
/// # Panics
/// This function does not panic.
///
/// # Logic
/// The handler:
/// - Initializes a counter to 0
/// - Accesses the energy cell and increments the counter if it is charged
/// - Wraps the counter inside an [AvailableEnergyCellResponse] message and returns it
#[allow(dead_code)]
fn handle_energy_cell_request(
    state: &PlanetState,
) -> Option<PlanetToExplorer> {
    let mut available_cells = 0;
    if state.cell(0).is_charged() {
        available_cells += 1;
    }
    Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells })
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::collections::HashSet;
    use std::{thread};
    use common_game::components::asteroid::Asteroid;
    use common_game::components::sunray::Sunray;
    use common_game::components::resource::{BasicResourceType, ComplexResourceType};
    use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};
    use crossbeam_channel::{unbounded, Receiver, Sender};
    use crate::planetAI::{handle_energy_cell_request, handle_supported_combination_request, handle_supported_resource_request, create_planet};



    fn planet_to_explorer_channel_creator() -> (Sender<PlanetToExplorer>, Receiver<PlanetToExplorer>) {
        let (planet_to_explorer_sender, planet_to_explorer_receiver): (Sender<PlanetToExplorer>, Receiver<PlanetToExplorer>) = unbounded();
        (planet_to_explorer_sender, planet_to_explorer_receiver)
    }


    fn explorer_to_planet_channels_creator() -> (Sender<ExplorerToPlanet>, Receiver<ExplorerToPlanet>) {
        let (explorer_to_planet_sender, explorer_to_planet_receiver): (Sender<ExplorerToPlanet>, Receiver<ExplorerToPlanet>) = unbounded();
        (explorer_to_planet_sender, explorer_to_planet_receiver)
    }

    fn orchestrator_to_planet_channels_creator() -> (Sender<OrchestratorToPlanet>, Receiver<OrchestratorToPlanet>) {
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver): (Sender<OrchestratorToPlanet>, Receiver<OrchestratorToPlanet>) = unbounded();
        (orchestrator_to_planet_sender, orchestrator_to_planet_receiver)
    }


    fn planet_to_orchestrator_channels_creator() -> (Sender<PlanetToOrchestrator>, Receiver<PlanetToOrchestrator>) {
        let (planet_to_orchestrator_sender, planet_to_orchestrator_receiver): (Sender<PlanetToOrchestrator>, Receiver<PlanetToOrchestrator>) = unbounded();
        (planet_to_orchestrator_sender, planet_to_orchestrator_receiver)
    }



    /// Assert that when the cells are not charged (which means as soon as the planet is created)
    /// the [Asteroid] destroys the [Planet].
    #[test]
    fn asteroid_with_uncharged_cell() {

        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Trace)
            .init();

        // ----------------- Channels and planet creation
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) = orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrator_sender, planet_to_orchestrator_receiver) = planet_to_orchestrator_channels_creator();
        let (_, explorer_to_planet_receiver) = explorer_to_planet_channels_creator();
        let mut planet = create_planet(
            orchestrator_to_planet_receiver,
            planet_to_orchestrator_sender,
            explorer_to_planet_receiver,
            2,
        );

        // ----------------- Spawn the thread:
        let _ = thread::spawn(move || {
            let _ = planet.run();
        });

        // ----------------- Make the planet start
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::StartPlanetAI);
        let _ = planet_to_orchestrator_receiver.recv();


        // ----------------- Send an asteroid
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::Asteroid(Asteroid::default()));
        let planet_response = planet_to_orchestrator_receiver.recv().unwrap();
        assert!(matches!( planet_response, PlanetToOrchestrator::AsteroidAck { .. } ));
        assert!(matches!( planet_response, PlanetToOrchestrator::AsteroidAck { planet_id: 2, rocket: None }));
    }


    /// Assert that when the cell is charged the [Asteroid] does not destroy the [Planet].
    #[test]
    fn test_asteroid_handler_with_charged_cell() {

        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Trace)
            .init();

        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) = orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrator_sender, planet_to_orchestrator_receiver) = planet_to_orchestrator_channels_creator();

        let (_, explorer_to_planet_receiver) = explorer_to_planet_channels_creator();

        let mut planet = create_planet(
            orchestrator_to_planet_receiver,
            planet_to_orchestrator_sender,
            explorer_to_planet_receiver,
            2,
        );

        // Spawn the thread:
        let _ = thread::spawn(move || {
            let _ = planet.run();
        });

        // Make the PlanetAI start
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::StartPlanetAI);
        let _ = planet_to_orchestrator_receiver.recv();


        // Send sunrays
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::Sunray(Sunray::default()));
        let sunrays_planet_response = planet_to_orchestrator_receiver.recv();
        assert!(matches!( sunrays_planet_response, Ok(PlanetToOrchestrator::SunrayAck { .. }) ), "Did not received a sunrays AKC");
        assert!(matches!(sunrays_planet_response, Ok(PlanetToOrchestrator::SunrayAck { planet_id: 2})));

        // Send the asteroid
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::Asteroid(Asteroid::default()));
        let planet_response = planet_to_orchestrator_receiver.recv();
        assert!(planet_response.is_ok(), "Error with the response of the planet once the Asteroid");

        let planet_response_msg = planet_response.unwrap();
        assert!(matches!( planet_response_msg, PlanetToOrchestrator::AsteroidAck { planet_id: 2,  rocket: _ }), "Planet answered with a different ID");
        assert!(matches!( planet_response_msg, PlanetToOrchestrator::AsteroidAck { planet_id: 2,  rocket: Some( _ ) }));
        assert!(matches!(planet_response_msg, PlanetToOrchestrator::AsteroidAck { .. } ), "The planet did not answer back with a AsteroidAck");

    }


    /// Send the rocket when the AI is not enabled
    #[test]
    fn test_rocket_with_disabled_ai() {

        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Trace)
            .init();

        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) = orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrator_sender, planet_to_orchestrator_receiver) = planet_to_orchestrator_channels_creator();

        let (_, explorer_to_planet_receiver) = explorer_to_planet_channels_creator();


        let mut planet = create_planet(

            orchestrator_to_planet_receiver,
            planet_to_orchestrator_sender ,
            explorer_to_planet_receiver,
            2
        );

        let _ = thread::spawn(move|| {
            planet.run()
        });

        // Start the AI (disabled by default)
        let _ = orchestrator_to_planet_sender.send( OrchestratorToPlanet::StartPlanetAI);
        let _ = planet_to_orchestrator_receiver.recv();



        // Shutdown the planet AI. Note that I should not wait for the response
        let _ = orchestrator_to_planet_sender.send( OrchestratorToPlanet::StopPlanetAI );
        let ai_shutdown_response = planet_to_orchestrator_receiver.recv();
        assert!(matches!(ai_shutdown_response, Ok(PlanetToOrchestrator::StopPlanetAIResult { .. })));

        // Send the asteroid
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::Asteroid(Asteroid::default()));
        let planet_response = planet_to_orchestrator_receiver.recv();
        assert!(matches!(planet_response, Ok(PlanetToOrchestrator::Stopped {..})));

    }


    /// Testing the start and stop of the AI.
    #[test]
    fn test_start_and_stop_planet_ai() {

        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Trace)
            .init();

        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) = orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrator_sender, planet_to_orchestrator_receiver) = planet_to_orchestrator_channels_creator();

        let (_, explorer_to_planet_receiver) = explorer_to_planet_channels_creator();


        let mut planet = create_planet(
            orchestrator_to_planet_receiver,
            planet_to_orchestrator_sender,
            explorer_to_planet_receiver,
            2,
        );


        let _ = thread::spawn(move|| {
            let _ = planet.run();
        } );

        // Send start AI message
        let _ = orchestrator_to_planet_sender.send( OrchestratorToPlanet::StartPlanetAI );
        let response = planet_to_orchestrator_receiver.recv();
        assert!( matches!(response, Ok(PlanetToOrchestrator::StartPlanetAIResult { .. })) , "Expected a StartPlanetAIResult!");
        assert!( matches!(response, Ok(PlanetToOrchestrator::StartPlanetAIResult { planet_id: 2 })) , "Expected a StartPlanetAIResult with id = 2!");


        // Send stop AI message
        let _ = orchestrator_to_planet_sender.send( OrchestratorToPlanet::StopPlanetAI );
        let response = planet_to_orchestrator_receiver.recv();
        assert!( matches!(response, Ok(PlanetToOrchestrator::StopPlanetAIResult { .. })) , "Expected a StopPlanetAIResult");
        assert!( matches!(response, Ok(PlanetToOrchestrator::StopPlanetAIResult { planet_id: 2 })), "Expected a StopPlanetAIResult with id = 2");

        // Again, send startAI
        let _ = orchestrator_to_planet_sender.send( OrchestratorToPlanet::StartPlanetAI );
        let response = planet_to_orchestrator_receiver.recv();
        assert!( matches!(response, Ok(PlanetToOrchestrator::StartPlanetAIResult { .. })) , "Expected a StartPlanetAIResult!");
        assert!( matches!(response, Ok(PlanetToOrchestrator::StartPlanetAIResult { planet_id: 2 })) , "Expected a StartPlanetAIResult with id = 2!");

    }

    #[test]
    fn test_unit_handle_supported_resource_request() {
        let (to_orchestrator_tx, _to_orchestrator_rx) = planet_to_orchestrator_channels_creator(); // Planet -> Orchestrator
        let (_from_orchestrator_tx, from_orchestrator_rx) = orchestrator_to_planet_channels_creator(); // Orchestrator -> Planet
        let (_to_explorer_tx, _to_explorer_rx) = planet_to_explorer_channel_creator(); // Planet -> Explorer
        let (_from_explorer_tx, from_explorer_rx) = explorer_to_planet_channels_creator(); // Explorer -> Planet

        let planet = create_planet(
            from_orchestrator_rx,
            to_orchestrator_tx,
            from_explorer_rx,
            2,
        );

        let result = handle_supported_resource_request(planet.generator());

        assert!(result.is_some());

        if let Some(PlanetToExplorer::SupportedResourceResponse { resource_list }) = result {
            let result_set: HashSet<BasicResourceType> = resource_list.into_iter().collect();
            let expected_set: HashSet<BasicResourceType> = vec![BasicResourceType::Carbon].into_iter().collect();
            assert_eq!(result_set, expected_set);
        } else {
            panic!("Expected SupportedResourceResponse variant");
        }
    }

    #[test]
    fn test_unit_handle_supported_combination_request() {
        let (to_orchestrator_tx, _to_orchestrator_rx) = planet_to_orchestrator_channels_creator(); // Planet -> Orchestrator
        let (_from_orchestrator_tx, from_orchestrator_rx) = orchestrator_to_planet_channels_creator(); // Orchestrator -> Planet
        let (_to_explorer_tx, _to_explorer_rx) = planet_to_explorer_channel_creator(); // Planet -> Explorer
        let (_from_explorer_tx, from_explorer_rx) = explorer_to_planet_channels_creator(); // Explorer -> Planet

        let planet = create_planet(
            from_orchestrator_rx,
            to_orchestrator_tx,
            from_explorer_rx,
            2,
        );

        let result = handle_supported_combination_request(planet.combinator());

        assert!(result.is_some());

        if let Some(PlanetToExplorer::SupportedCombinationResponse { combination_list }) = result {
            let result_set: HashSet<ComplexResourceType> = combination_list.into_iter().collect();
            let expected_set: HashSet<ComplexResourceType> = vec![
                ComplexResourceType::Diamond,
                ComplexResourceType::Life,
                ComplexResourceType::AIPartner,
                ComplexResourceType::Dolphin,
                ComplexResourceType::Robot,
                ComplexResourceType::Water
            ].into_iter().collect();
            assert_eq!(result_set, expected_set);
        } else {
            panic!("Expected SupportedCombinationResponse variant");
        }
    }

    #[test]
    fn test_integration_handle_orchestrator_msg_sunray_and_handle_energy_cell_request_charge() {
        let (to_orchestrator_tx, _to_orchestrator_rx) = planet_to_orchestrator_channels_creator(); // Planet -> Orchestrator
        let (from_orchestrator_tx, from_orchestrator_rx) = orchestrator_to_planet_channels_creator(); // Orchestrator -> Planet
        let (_to_explorer_tx, _to_explorer_rx) = planet_to_explorer_channel_creator(); // Planet -> Explorer
        let (_from_explorer_tx, from_explorer_rx) = explorer_to_planet_channels_creator(); // Explorer -> Planet

        let planet = Arc::new(Mutex::new(create_planet(
            from_orchestrator_rx,
            to_orchestrator_tx,
            from_explorer_rx,
            2,
        )));

        let planet_for_thread = Arc::clone(&planet);

        let thread_var = thread::spawn(move || {
            let _ = planet_for_thread.lock().unwrap().run();
        });

        let _ = from_orchestrator_tx.send(OrchestratorToPlanet::StartPlanetAI);
        let _ = from_orchestrator_tx.send(OrchestratorToPlanet::Sunray(Sunray::default()));
        let _ = from_orchestrator_tx.send(OrchestratorToPlanet::StopPlanetAI);

        drop(from_orchestrator_tx);
        let _ = thread_var.join();

        let planet_guard = planet.lock().unwrap();
        let result = handle_energy_cell_request(planet_guard.state());

        assert!(result.is_some());

        if let Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells }) = result {
            assert_eq!(available_cells, 1);
        } else {
            panic!("Expected AvailableEnergyCellResponse");
        }
    }

    #[test]
    fn test_unit_handle_energy_cell_request_discharge() {
        let (to_orchestrator_tx, _to_orchestrator_rx) = planet_to_orchestrator_channels_creator(); // Planet -> Orchestrator
        let (_from_orchestrator_tx, from_orchestrator_rx) = orchestrator_to_planet_channels_creator(); // Orchestrator -> Planet
        let (_to_explorer_tx, _to_explorer_rx) = planet_to_explorer_channel_creator(); // Planet -> Explorer
        let (_from_explorer_tx, from_explorer_rx) = explorer_to_planet_channels_creator(); // Explorer -> Planet

        let planet = create_planet(
            from_orchestrator_rx,
            to_orchestrator_tx,
            from_explorer_rx,
            2,
        );

        let result = handle_energy_cell_request(planet.state());

        assert!(result.is_some());

        if let Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells }) = result {
            let expected_usize = 0;
            assert_eq!(available_cells, expected_usize);
        } else {
            panic!("Expected SupportedCombinationResponse variant");
        }
    }
}

/// Wrapper fot the loggin module defined in the common crate.
mod logging_wrapper {
    use common_game::components::planet::PlanetState;
    use common_game::logging::{ActorType, Channel, EventType, LogEvent, Payload};


    fn log_message(
        sender_id: impl Into<u64>,
        receiver_type: ActorType,
        receiver_id: impl Into<String>,
        event_type: EventType,
        channel: Channel,
        payload: Vec<String>,
    ) {

        // Create the planet object
        let mut payload_object = Payload::new();

        payload.iter().enumerate().map(|(index, inserted_string)| {
            match channel {
                Channel::Error => (format!("Error #{} detail:", index), inserted_string),
                Channel::Info => (format!("Info #{} detail:", index), inserted_string),
                Channel::Debug => (format!("Debug #{} detail:", index), inserted_string),
                Channel::Warning => (format!("Warning #{} detail:", index), inserted_string),
                Channel::Trace => (format!("Trace #{} detail:", index), inserted_string)
            }
        }).for_each(|(key_val, string_req)| {
            payload_object.insert(key_val, string_req.to_string());
        });

        LogEvent::new(
            ActorType::Planet,
            sender_id,
            receiver_type,
            receiver_id,
            event_type,
            channel,
            payload_object
        ).emit();
    }

    fn log_message_with_key(
        sender_id: impl Into<u64>,
        receiver_type: ActorType,
        receiver_id: impl Into<String>,
        event_type: EventType,
        channel: Channel,
        payload: Vec<(String, String)>,
    ) {
        // Create the planet object
        let mut payload_object = Payload::new();

        payload.iter().for_each(
            |(payload_key, payload_content)| {
                payload_object.insert( payload_key.to_string(), payload_content.to_string() );
            }
        );

        LogEvent::new(
            ActorType::Planet,
            sender_id,
            receiver_type,
            receiver_id,
            event_type,
            channel,
            payload_object
        ).emit();
    }

    /*
        Macro to build the specialize loggin for each channel
    */
    macro_rules! specialize_channel_func {
        ( $( $Enum:ident => $name:literal ),* ) => {
            $(
                paste::paste!(
                    pub fn [<log_for_channel_ $name:lower>](
                        sender_id: impl Into<u64>,
                        receiver_type: ActorType,
                        receiver_id: impl Into<String>,
                        event_type: EventType,
                        payload: Vec<String>,
                    ) {
                        log_message(
                            sender_id.into(),
                            receiver_type,
                            receiver_id.into(),
                            event_type,
                            Channel::$Enum,
                            payload
                        )
                    }

                    pub fn [<log_for_channel_with_key_ $name:lower>] (
                        sender_id: impl Into<u64>,
                        receiver_type: ActorType,
                        receiver_id: impl Into<String>,
                        event_type: EventType,
                        payload: Vec<(String, String)>,
                    ) {
                        log_message_with_key(
                            sender_id.into(),
                            receiver_type,
                            receiver_id.into(),
                            event_type,
                            Channel::$Enum,
                            payload
                        )
                    }
                );
            )*
        };
    }


    // Define scope of the macro
    specialize_channel_func!(
        Error => "Error",
        Warning => "Warning",
        Info => "Info",
        Debug => "Debug",
        Trace => "Trace"
    );


    pub fn drop_planet_state_fields_as_vector(planet_state: &PlanetState) -> Vec<(String, String)> {
        let stringify_rocket = |option_rocket: bool| -> String  {match option_rocket { true => "Some(rocket)".to_string(), false => "None".to_string() } };
        vec!(
            ("id".to_string(), format!("{:?}", planet_state.id())),
            ("energy_cell".to_string(), format!("Vec: {{ {} }}", planet_state.cell(0).is_charged()) ),
            ( "rocket".to_string(), format!("{}", stringify_rocket(planet_state.has_rocket())) )
        )
    }

    pub fn _drop_energy_cell_state_fields_as_vector(planet_state: &PlanetState) -> Vec<(String, String)> {
        vec![drop_planet_state_fields_as_vector(planet_state).remove(1)]
    }

    pub fn _drop_energy_rocket_state_fields_as_vector(planet_state: &PlanetState) -> Vec<(String, String)> {
        vec![drop_planet_state_fields_as_vector(planet_state).remove(2)]
    }

}