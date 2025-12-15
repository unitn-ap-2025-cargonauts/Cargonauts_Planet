//! # Cargonauts planet AI Module
//!
//! This module contains the implementation of the [PlanetAI] trait for the
//! Cargonauts planet. It defines how the planet handle messages from the
//! Orchestrator and the Explorer.
//!
//! Each handler is defined as a standalone function to keep the logic modular and clean.

use common_game::components::planet::*;
use common_game::components::resource::*;
use common_game::components::rocket::Rocket;
use common_game::logging::{ActorType, EventType};
use common_game::protocols::messages::*;
use crossbeam_channel::{Receiver, Sender};
use paste::paste;
use crate::logging_wrapper as logging_wrapper;

//For docs
#[allow(unused_imports)]
use common_game::components::energy_cell::EnergyCell;
use common_game::components::sunray::Sunray;

struct CargonautsPlanet;

#[allow(rustdoc::private_intra_doc_links)]
/// Creates a new [`Planet`] instance set for be a Cargonauts planet.
///
/// # Parameters
/// - `rx_orchestrator`: Receiver for messages sent from the orchestrator to the planet
///   ([`OrchestratorToPlanet`]).
/// - `tx_orchestrator`: Sender for messages sent from the planet to the orchestrator
///   ([`PlanetToOrchestrator`]).
/// - `rx_explorer`: Receiver for messages sent from the explorer to the planet
///   ([`ExplorerToPlanet`]).
/// - `planet_id`: Unique identifier assigned to the created planet.
///
/// # Returns
/// A fully initialized [`Planet`] instance configured with:
/// - planet type [`PlanetType::C`]
/// - A default implementation of [`CargonautsPlanet`] AI
/// - Support for generating the basic resource [`BasicResourceType::Carbon`]
/// - Support for crafting complex resources:
///   [`Diamond`], [`Life`], [`AIPartner`], [`Dolphin`], [`Robot`], [`Water`]
///
/// # Panics
/// This function **panics** if the planet creation fails.
///
/// The panic message is:
/// **"planet creation error!"**
#[allow(dead_code)]
pub fn create_planet(
    rx_orchestrator: Receiver<OrchestratorToPlanet>,
    tx_orchestrator: Sender<PlanetToOrchestrator>,
    rx_explorer: Receiver<ExplorerToPlanet>,
    planet_id: u32,
) -> Planet {
    let planet = Planet::new(
        planet_id,
        PlanetType::C,
        Box::new(CargonautsPlanet),
        vec![BasicResourceType::Carbon],
        vec![
            ComplexResourceType::Diamond,
            ComplexResourceType::Life,
            ComplexResourceType::AIPartner,
            ComplexResourceType::Dolphin,
            ComplexResourceType::Robot,
            ComplexResourceType::Water,
        ],
        (rx_orchestrator, tx_orchestrator),
        rx_explorer,
    );
    assert!(planet.is_ok(), "Planet creation error!");
    planet.unwrap()
}

impl Default for CargonautsPlanet {
    fn default() -> Self {
        Self
    }
}

impl PlanetAI for CargonautsPlanet {

    // === OrchestratorToPlanet Handler ====================================================================
    /// TODO Handles messages sent by the Orchestrator to the Planet.
    ///
    /// # Parameters
    /// - `state`: Mutable reference to the current state of the planet.
    /// - `_generator`: Reference to the generator (unused in this handler).
    /// - `_combinator`: Reference to the combinator (unused in this handler).
    /// - `msg`: The [`OrchestratorToPlanet`] message to handle.
    ///
    /// # Returns
    /// `Option<PlanetToOrchestrator>`
    ///
    /// Returns a response message to the Orchestrator if the processed message requires one,
    /// or `None` otherwise.
    ///
    /// # Logic
    /// The handler processes specific messages that modify the state or request information:
    /// - **[`OrchestratorToPlanet::Sunray`]**: Charges the planet's energy cell and returns a [`PlanetToOrchestrator::SunrayAck`].
    /// - **[`OrchestratorToPlanet::InternalStateRequest`]**: Serializes the current planet state and returns it via [`PlanetToOrchestrator::InternalStateResponse`].
    /// - **Other messages**: Returns `None`. Messages such as `Start`, `Stop`, or `Asteroid` are handled by the main `run` loop or are ignored in this context.
    fn handle_sunray(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, sunray: Sunray) {
        let planet_id = state.id();

        logging_wrapper::log_for_channel_with_key_debug( //TODO adapt log
            planet_id,
            ActorType::SelfActor,
            "0",
            EventType::InternalPlanetAction,
            vec![
                (
                    "Debug detail".to_string(),
                    "Called handle_orchestrator_msg".to_string(),
                ),
                // We cannot log 'msg' here easily if it is moved into the match,
                // so we log the specific variant inside the match arms.
            ],
        );

        //LOG trace: Processing Sunray
        logging_wrapper::log_for_channel_trace(
            planet_id,
            ActorType::SelfActor,
            0.to_string(),
            EventType::InternalPlanetAction,
            vec!["Received Sunray, attempting to charge cell".to_string()],
        );

        let _ = state.charge_cell(sunray);

        //LOG info: SunrayAck
        logging_wrapper::log_for_channel_with_key_info(
            planet_id,
            ActorType::Orchestrator,
            0.to_string(),
            EventType::MessagePlanetToOrchestrator,
            vec![
                (
                    "msg_enum".to_string(),
                    "SunrayAck".to_string(),
                ),
                (
                    "action".to_string(),
                    "Cell charged".to_string(),
                ),
            ],
        );
    }

    /// Handler for the [OrchestratorToPlanet::Asteroid] message, it returns `None` or `Some([Rocket])` based on the rules of the
    /// [Planet] or the availability of [Rocket] on the planet.
    ///
    /// More precisely, it returns `None` if:
    /// - The [Planet] can not create any [Rocket] because of its rules.
    /// - The [Planet] can not crate any [Rocket] because it has no charged [EnergyCell].
    ///
    /// It returns `Some(Rocket)` if:
    /// - The [Planet]'s rules allow it to do so and it already has a [Rocket] that can be used.
    /// - The [Planet]'s rules allow it to do so and it was able to build a [Rocket] when [OrchestratorToPlanet::Asteroid]
    ///   message was delivered to it.
    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _: &Generator,
        _: &Combinator,
    ) -> Option<Rocket> {
        if state.has_rocket() {
            let before_take_rocket = logging_wrapper::drop_planet_state_as_string(state);

            let rocket = state.take_rocket();

            match rocket {
                Some(taken_rocket) => {
                    logging_wrapper::log_for_channel_info(
                        state.id(),
                        ActorType::SelfActor,
                        0.to_string(),
                        EventType::InternalPlanetAction,
                        vec![
                            "planet received an Asteroid and has the rocket to deflect it."
                                .to_string(),
                        ],
                    );

                    logging_wrapper::log_for_channel_with_key_trace(
                        state.id(),
                        ActorType::SelfActor,
                        0.to_string(),
                        EventType::InternalPlanetAction,
                        vec![
                            (
                                "Traced message:".to_string(),
                                "Rocket successfully deflected".to_string(),
                            ),
                            (
                                "State before Rocket usage: ".to_string(),
                                format!("{:?}", before_take_rocket),
                            ),
                            (
                                "State after Rocket usage: ".to_string(),
                                format!(
                                    "{:?}",
                                    logging_wrapper::drop_planet_state_as_string(state)
                                ),
                            ),
                        ],
                    );

                    logging_wrapper::log_for_channel_debug(
                        state.id(),
                        ActorType::SelfActor,
                        0.to_string(),
                        EventType::InternalPlanetAction,
                        vec!["Asteroid handled".to_string()],
                    );
                    Some(taken_rocket)
                }
                None => {
                    // Strange behavior: the .has_rocket failed! Log the error
                    logging_wrapper::log_for_channel_with_key_error(
                        state.id(),
                        ActorType::SelfActor,
                        0.to_string(),
                        EventType::InternalPlanetAction,
                        vec![
                            (
                                "Error description".to_string(),
                                "take_rocket() function failed: returned None".to_string(),
                            ),
                            (
                                "planet state".to_string(),
                                format!(
                                    "{:?}",
                                    logging_wrapper::drop_planet_state_as_string(state)
                                ),
                            ),
                        ],
                    );

                    None
                }
            }
        } else {
            //The Rocket is not available, try to build it
            let charged_cell_position = state
                .cells_iter()
                .position(|cell_ref| cell_ref.is_charged());
            if let Some(charged_cell_position_result) = charged_cell_position {
                // There is a charged cell, therefore a Rocket can be built

                // Build the rocket
                let created_rocket_result = state.build_rocket(charged_cell_position_result);

                // Check the Result
                match created_rocket_result {
                    Ok(_) => {
                        // Successfully created the rocket

                        logging_wrapper::log_for_channel_info(
                            state.id(),
                            ActorType::SelfActor,
                            0.to_string(),
                            EventType::InternalPlanetAction,
                            vec!["planet received an Asteroid and has just created a rocket to deflect it. Proceeding".to_string()]
                        );

                        logging_wrapper::log_for_channel_with_key_trace(
                            state.id(),
                            ActorType::SelfActor,
                            0.to_string(),
                            EventType::InternalPlanetAction,
                            vec![
                                (
                                    "Traced message:".to_string(),
                                    "Rocket successfully deflected".to_string(),
                                ),
                                (
                                    "State after Rocket creation: ".to_string(),
                                    format!(
                                        "{:?}",
                                        logging_wrapper::drop_planet_state_as_string(state)
                                    ),
                                )
                            ],
                        );



                        let rocket_created =  state.take_rocket();

                        // Give ownership to the orchestrator
                        logging_wrapper::log_for_channel_debug(
                            state.id(),
                            ActorType::SelfActor,
                            0.to_string(),
                            EventType::InternalPlanetAction,
                            vec!["Asteroid handled".to_string()],
                        );
                        rocket_created
                    }
                    Err(string_error) => {
                        // Technically the rocket could be built but something went wrong; logging the error
                        logging_wrapper::log_for_channel_with_key_error(
                            state.id(),
                            ActorType::SelfActor,
                            0.to_string(),
                            EventType::InternalPlanetAction,
                            vec![
                                (
                                    "Error description".to_string(),
                                    format!(
                                        "build_rocket() function failed. Error reason: {}",
                                        string_error
                                    )
                                    .to_string(),
                                ),
                                (
                                    "planet state".to_string(),
                                    logging_wrapper::drop_planet_state_as_string(state),
                                ),
                            ],
                        );
                        None
                    }
                }
            } else {
                // Rocket can not be built
                logging_wrapper::log_for_channel_info(
                    state.id(),
                    ActorType::SelfActor,
                    0.to_string(),
                    EventType::InternalPlanetAction,
                    vec!["planet received an Asteroid and cannot defend itself and it is being destroyed".to_string()],
                );
                logging_wrapper::log_for_channel_with_key_trace(
                    state.id(),
                    ActorType::SelfActor,
                    0.to_string(),
                    EventType::InternalPlanetAction,
                    vec![
                        (
                        "State of planet before being destroyed".to_string(),
                        logging_wrapper::drop_planet_state_as_string(state),
                    )],
                );

                None
            }
        }
    }

    /// TODO Handles messages sent by the Orchestrator to the Planet.
    ///
    /// # Parameters
    /// - `state`: Mutable reference to the current state of the planet.
    /// - `_generator`: Reference to the generator (unused in this handler).
    /// - `_combinator`: Reference to the combinator (unused in this handler).
    /// - `msg`: The [`OrchestratorToPlanet`] message to handle.
    ///
    /// # Returns
    /// `Option<PlanetToOrchestrator>`
    ///
    /// Returns a response message to the Orchestrator if the processed message requires one,
    /// or `None` otherwise.
    ///
    /// # Logic
    /// The handler processes specific messages that modify the state or request information:
    /// - **[`OrchestratorToPlanet::Sunray`]**: Charges the planet's energy cell and returns a [`PlanetToOrchestrator::SunrayAck`].
    /// - **[`OrchestratorToPlanet::InternalStateRequest`]**: Serializes the current planet state and returns it via [`PlanetToOrchestrator::InternalStateResponse`].
    /// - **Other messages**: Returns `None`. Messages such as `Start`, `Stop`, or `Asteroid` are handled by the main `run` loop or are ignored in this context.
    fn handle_internal_state_req(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator) -> DummyPlanetState {
        let planet_id = state.id();

        logging_wrapper::log_for_channel_with_key_debug( //TODO adapt log
            planet_id,
            ActorType::SelfActor,
            "0",
            EventType::InternalPlanetAction,
            vec![
                (
                    "Debug detail".to_string(),
                    "Called handle_orchestrator_msg".to_string(),
                ),
                // We cannot log 'msg' here easily if it is moved into the match,
                // so we log the specific variant inside the match arms.
            ],
        );

        //LOG trace: Processing InternalStateRequest
        logging_wrapper::log_for_channel_trace(
            planet_id,
            ActorType::SelfActor,
            0.to_string(),
            EventType::InternalPlanetAction,
            vec!["Received InternalStateRequest, gathering state".to_string()],
        );

        let dummy_state = state.to_dummy();

        //LOG info: InternalStateResponse
        logging_wrapper::log_for_channel_with_key_info(
            planet_id,
            ActorType::Orchestrator,
            0.to_string(),
            EventType::MessagePlanetToOrchestrator,
            vec![
                (
                    "msg_enum".to_string(),
                    "InternalStateResponse".to_string(),
                ),
                (
                    "state_dump".to_string(),
                    format!("{:?}", &dummy_state)
                ),
            ],
        );

        dummy_state
    }

    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id } => {
                handle_supported_resource_request(state.id(), explorer_id, generator)
            }
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                handle_supported_combination_request(state.id(), explorer_id, combinator)
            }
            ExplorerToPlanet::GenerateResourceRequest {
                explorer_id,
                resource,
            } => handle_generate_resource_request(explorer_id, state, generator, resource),
            ExplorerToPlanet::CombineResourceRequest { explorer_id, msg } => {
                handle_combine_resource_request(explorer_id, state, combinator, msg)
            }
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id } => {
                handle_energy_cell_request(explorer_id, state)
            }
        }
    }

    fn on_explorer_arrival(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, explorer_id: u32) {
        todo!()
    }

    fn on_explorer_departure(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, explorer_id: u32) {
        todo!()
    }

    /// This method will be invoked when a [OrchestratorToPlanet::StartPlanetAI]
    /// is received, but **only if** the planet is currently in a *stopped* state.
    ///
    /// Start messages received when planet is already running are **ignored**.
    fn on_start(&mut self, state: &PlanetState, generator: &Generator, combinator: &Combinator) {
        logging_wrapper::log_for_channel_with_key_debug(
            state.id(),
            ActorType::SelfActor,
            0.to_string(),
            EventType::InternalPlanetAction,
            vec![(
                "State of the planet upon receipt of the StartPlanetAI message".to_string(),
                format!("{:?}", logging_wrapper::drop_planet_state_as_string(state)),
            )],
        );
    }

    /// This method will be invoked when a [OrchestratorToPlanet::StopPlanetAI]
    /// is received, but **only if** the planet is currently in a *running* state.
    ///
    fn on_stop(&mut self, state: &PlanetState, generator: &Generator, combinator: &Combinator) {
        logging_wrapper::log_for_channel_with_key_info(
            state.id(),
            ActorType::SelfActor,
            0.to_string(),
            EventType::InternalPlanetAction,
            vec![(
                "State of the planet upon receipt of the StopPlanetAi message".to_string(),
                format!("{:?}", logging_wrapper::drop_planet_state_as_string(state)),
            )],
        );
    }
}

// === ExplorerToPlanet Handler ====================================================================
/// This handler returns a [PlanetToExplorer::SupportedResourceResponse] message that wrap the list of basic resources
/// that the planet can currently generate
///
/// # Parameters
/// - `generator`: Reference to the planet's generator
///
/// # Returns
/// `Option<PlanetToExplorer>`
///
/// `Some(PlanetToExplorer::SupportedResourceResponse)` if successful.
///
/// # Logic
/// The planet can craft basic resources, so the handler:
/// - Get the set of available basic resource from the planet generator
/// - Wrap the set in a [PlanetToExplorer::SupportedResourceResponse] message and return it
#[allow(dead_code)]
fn handle_supported_resource_request(
    planet_id: u32,
    explorer_id: u32,
    generator: &Generator,
) -> Option<PlanetToExplorer> {
    //LOG debug: "Called handle_supported_resource_request" + parameters
    logging_wrapper::log_for_channel_with_key_debug(
        planet_id,
        ActorType::SelfActor,
        "0",
        EventType::InternalPlanetAction,
        vec![
            (
                "Debug detail".to_string(),
                "Called handle_supported_resource_request".to_string(),
            ),
            ("generator".to_string(), format!("{:?}", generator)),
        ],
    );
    let resource_list = generator.all_available_recipes();
    //LOG info: "SupportedResourceResponse"
    logging_wrapper::log_for_channel_with_key_info(
        planet_id,
        ActorType::Explorer,
        explorer_id.to_string(),
        EventType::MessagePlanetToExplorer,
        vec![
            (
                "msg_enum".to_string(),
                "SupportedResourceResponse".to_string(),
            ),
            ("resource_list".to_string(), format!("{:?}", &resource_list)),
        ],
    );
    //LOG debug: "Exit handle_supported_resource_request" + value return
    logging_wrapper::log_for_channel_with_key_debug(
        planet_id,
        ActorType::SelfActor,
        "0",
        EventType::InternalPlanetAction,
        vec![
            (
                "Debug detail".to_string(),
                "Exit handle_supported_resource_request".to_string(),
            ),
            (
                "return".to_string(),
                format!(
                    "Some(PlanetToExplorer::SupportedResourceResponse{{{:?}}})",
                    &resource_list
                ),
            ),
        ],
    );
    Some(PlanetToExplorer::SupportedResourceResponse { resource_list })
}

/// This handler returns a [PlanetToExplorer::SupportedCombinationResponse] message that wrap the list of complex resources
/// that the planet can currently generate
///
/// # Parameters
/// - `combinator`: Reference to the planet's combinator
///
/// # Returns
/// `Option<PlanetToExplorer>`
///
/// `Some(PlanetToExplorer::SupportedCombinationResponse)` if successful.
///
/// # Logic
/// The planet can craft complex resources, so the handler:
/// - Get the set of available complex resource from the planet combinator
/// - Wrap the set in a [PlanetToExplorer::SupportedCombinationResponse] message and return it
#[allow(dead_code)]
fn handle_supported_combination_request(
    planet_id: u32,
    explorer_id: u32,
    combinator: &Combinator,
) -> Option<PlanetToExplorer> {
    //LOG debug: "Called handle_supported_combination_request" + parameters
    logging_wrapper::log_for_channel_with_key_debug(
        planet_id,
        ActorType::SelfActor,
        "0",
        EventType::InternalPlanetAction,
        vec![
            (
                "Debug detail".to_string(),
                "Called handle_supported_combination_request".to_string(),
            ),
            ("combinator".to_string(), format!("{:?}", combinator)),
        ],
    );
    let combination_list = combinator.all_available_recipes();
    //LOG info: "SupportedCombinationResponse"
    logging_wrapper::log_for_channel_with_key_info(
        planet_id,
        ActorType::Explorer,
        explorer_id.to_string(),
        EventType::MessagePlanetToExplorer,
        vec![
            (
                "msg_enum".to_string(),
                "SupportedCombinationResponse".to_string(),
            ),
            (
                "combination_list".to_string(),
                format!("{:?}", &combination_list),
            ),
        ],
    );
    //LOG debug: "Exit handle_supported_combination_request" + value return
    logging_wrapper::log_for_channel_with_key_debug(
        planet_id,
        ActorType::SelfActor,
        "0",
        EventType::InternalPlanetAction,
        vec![
            (
                "Debug detail".to_string(),
                "Exit handle_supported_combination_request".to_string(),
            ),
            (
                "return".to_string(),
                format!(
                    "Some(PlanetToExplorer::SupportedCombinationResponse{{{:?}}})",
                    &combination_list
                ),
            ),
        ],
    );
    Some(PlanetToExplorer::SupportedCombinationResponse { combination_list })
}

/// This handler processes a request to generate a basic resource using the planet's generator,
/// if energy is available.
/// It returns a [PlanetToExplorer::GenerateResourceResponse] message containing the generated resource.
///
/// # Parameters
/// - `state`: Mutable reference to the planet state.
/// - `generator`: Reference to the planet's generator.
/// - `req_resource`: The type of basic resource the explorer is requesting to generate.
///
/// # Returns
/// `Option<PlanetToExplorer>`
///
/// `Some(PlanetToExplorer::GenerateResourceResponse)` containing:
/// - `Some(BasicResource::Carbon)` on successful generation.
/// - `None` if the planet has no charged energy cell.
///
/// # Errors
/// - If the generator reports an unexpected error while crafting.
///
/// # Logic
/// - Retrieve the energy cell and check if it is charged
/// - If charged:
///     - Attempt to generate the requested basic resource via the generator
///     - Wrap the generated resource in a [PlanetToExplorer::GenerateResourceResponse] message and return it.
/// - Else:
///     - Wrap a `None` in a [PlanetToExplorer::GenerateResourceResponse] message and return it.
#[allow(dead_code)]
fn handle_generate_resource_request(
    explorer_id: u32,
    state: &mut PlanetState,
    generator: &Generator,
    req_resource: BasicResourceType,
) -> Option<PlanetToExplorer> {
    let planet_id = state.id();
    //LOG debug: "Called handle_generate_resource_request" + parameters
    logging_wrapper::log_for_channel_with_key_debug(
        planet_id,
        ActorType::SelfActor,
        "0",
        EventType::InternalPlanetAction,
        logging_wrapper::append_info_to_state(
            state,
            vec![
                (
                    "Debug detail".to_string(),
                    "Called handle_generate_resource_request".to_string(),
                ),
                ("generator".to_string(), format!("{:?}", generator)),
                ("req_resource".to_string(), format!("{:?}", &req_resource)),
            ],
        ),
    );
    let mut resource: Option<BasicResource> = None;
    let energy_cell = state.cell_mut(0);
    let is_charged = energy_cell.is_charged();
    //LOG trace: "Checking energy cell charge state: {is_charged:?}"
    logging_wrapper::log_for_channel_trace(
        planet_id,
        ActorType::SelfActor,
        explorer_id.to_string(),
        EventType::InternalPlanetAction,
        vec![format!(
            "Checking energy cell charge state: {}",
            if is_charged { "charged" } else { "discharged" }
        )],
    );
    if is_charged {
        match req_resource {
            BasicResourceType::Carbon => {
                //LOG trace: "Attempting to generate Carbon resource"
                logging_wrapper::log_for_channel_trace(
                    planet_id,
                    ActorType::SelfActor,
                    explorer_id.to_string(),
                    EventType::InternalPlanetAction,
                    vec!["Attempting to generate Carbon".to_string()],
                );
                match generator.make_carbon(energy_cell) {
                    Ok(r) => {
                        //LOG info: "Successfully generated Carbon resource"
                        logging_wrapper::log_for_channel_info(
                            planet_id,
                            ActorType::SelfActor,
                            explorer_id.to_string(),
                            EventType::InternalPlanetAction,
                            vec!["Successfully generated Carbon resource".to_string()],
                        );
                        resource = Some(BasicResource::Carbon(r))
                    }
                    Err(e) => {
                        //LOG error: "Failed to generate Carbon: {e:?}" + state
                        logging_wrapper::log_for_channel_with_key_error(
                            planet_id,
                            ActorType::SelfActor,
                            "0",
                            EventType::InternalPlanetAction,
                            logging_wrapper::append_info_to_state(
                                state,
                                vec![
                                    (
                                        "Error detail".to_string(),
                                        "Failed to generate Carbon".to_string(),
                                    ),
                                    ("error_msg".to_string(), format!("{:?}", &e)),
                                ],
                            ),
                        );
                    }
                }
            }
            _ => {
                //LOG error: "Unexpected resource type requested"
                logging_wrapper::log_for_channel_with_key_error(
                    planet_id,
                    ActorType::SelfActor,
                    "0",
                    EventType::InternalPlanetAction,
                    logging_wrapper::append_info_to_state(
                        state,
                        vec![
                            (
                                "Error detail".to_string(),
                                "Unexpected resource type requested".to_string(),
                            ),
                            ("req_resource".to_string(), format!("{:?}", &req_resource)),
                        ],
                    ),
                );
            }
        }
    } else {
        //LOG info: "Uncharged energy cell. Cannot generate resource Carbon"
        logging_wrapper::log_for_channel_info(
            planet_id,
            ActorType::SelfActor,
            explorer_id.to_string(),
            EventType::InternalPlanetAction,
            vec!["Uncharged energy cell. Cannot generate resource Carbon".to_string()],
        );
    }
    //LOG info: "GenerateResourceResponse"
    logging_wrapper::log_for_channel_with_key_info(
        planet_id,
        ActorType::Explorer,
        explorer_id.to_string(),
        EventType::MessagePlanetToExplorer,
        vec![
            (
                "msg_enum".to_string(),
                "GenerateResourceResponse".to_string(),
            ),
            ("resource".to_string(), format!("{:?}", &resource)),
        ],
    );
    //LOG debug: "Exit handle_generate_resource_request" + return
    logging_wrapper::log_for_channel_with_key_debug(
        planet_id,
        ActorType::SelfActor,
        "0",
        EventType::InternalPlanetAction,
        vec![
            (
                "Debug detail".to_string(),
                "Exit handle_generate_resource_request".to_string(),
            ),
            (
                "return".to_string(),
                format!(
                    "Some(PlanetToExplorer::GenerateResourceResponse{{{:?}}})",
                    &resource
                ),
            ),
        ],
    );
    Some(PlanetToExplorer::GenerateResourceResponse { resource })
}

macro_rules! generate_complex_resource {
    ($planet_id:expr, $explorer_id:expr, $combinator:expr, $cell:expr, $msg:expr, { $( $complex_resource:ident ( $r1:ident, $r2:ident ) ),* $(,)? }) => {{
        paste! {
            match $msg {
                $(
                    ComplexResourceRequest::$complex_resource($r1, $r2) => {
                        //LOG trace: "Attempting combination for {complex_resource}"
                        logging_wrapper::log_for_channel_trace(
                            $planet_id,
                            ActorType::SelfActor,
                            $explorer_id.to_string(),
                            EventType::InternalPlanetAction,
                            vec![format!("Attempting combination for {}", stringify!($complex_resource))]
                        );
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
/// It returns a [PlanetToExplorer::CombineResourceResponse] message containing the newly
/// crafted complex resource.
///
/// # Parameters
/// - `state`: Mutable reference to the planet state.
/// - `combinator`: Reference to the planet's combinator.
/// - `msg`: A [ComplexResourceRequest] specifying which complex resource the
///   explorer want to craft, with the ingredients required.
///
/// # Returns
/// `Option<PlanetToExplorer>`
///
/// `Some(PlanetToExplorer::CombineResourceResponse)` containing:
/// - `Ok(ComplexResource::X)` if the combination succeeds.
/// - `Err((String, GenericResource, GenericResource))` if the planet has no charged energy cell.
///
/// # Errors
/// - If the combinator reports an unexpected error while crafting.
///
/// # Logic
/// - Retrieve the energy cell and check if it is charged.
/// - If not charged:
///     - Wrap a `Err` with the passed [GenericResource] in a [PlanetToExplorer::CombineResourceResponse] message and return it.
/// - Else:
///     - Attempt to combine the provided ingredients using the combinator.
///     - Wrap the produced resource in a [PlanetToExplorer::CombineResourceResponse] message and return it.
#[allow(dead_code)]
fn handle_combine_resource_request(
    explorer_id: u32,
    state: &mut PlanetState,
    combinator: &Combinator,
    msg: ComplexResourceRequest,
) -> Option<PlanetToExplorer> {
    let planet_id = state.id();
    //LOG debug: "Called handle_combine_resource_request" + parameters
    logging_wrapper::log_for_channel_with_key_debug(
        state.id(),
        ActorType::SelfActor,
        "0",
        EventType::InternalPlanetAction,
        logging_wrapper::append_info_to_state(
            state,
            vec![
                (
                    "Debug detail".to_string(),
                    "Called handle_combine_resource_request".to_string(),
                ),
                ("combinator".to_string(), format!("{:?}", combinator)),
                ("msg".to_string(), format!("{:?}", &msg)),
            ],
        ),
    );
    let energy_cell = state.cell_mut(0);
    let is_charged = energy_cell.is_charged();
    //LOG trace: "Checking energy cell charge state: {is_charged:?}"
    logging_wrapper::log_for_channel_trace(
        planet_id,
        ActorType::SelfActor,
        explorer_id.to_string(),
        EventType::InternalPlanetAction,
        vec![format!(
            "Checking energy cell charge state: {}",
            if is_charged { "charged" } else { "discharged" }
        )],
    );
    if !is_charged {
        //LOG info: "Uncharged energy cell. Cannot combine resources"
        logging_wrapper::log_for_channel_info(
            state.id(),
            ActorType::SelfActor,
            explorer_id.to_string(),
            EventType::InternalPlanetAction,
            vec!["Uncharged energy cell. Cannot combine resources".to_string()],
        );
        let (r1, r2) = match msg {
            ComplexResourceRequest::Diamond(c1, c2) => (c1.to_generic(), c2.to_generic()),
            ComplexResourceRequest::Life(w, c) => (w.to_generic(), c.to_generic()),
            ComplexResourceRequest::Water(h, o) => (h.to_generic(), o.to_generic()),
            ComplexResourceRequest::Robot(s, l) => (s.to_generic(), l.to_generic()),
            ComplexResourceRequest::Dolphin(w, l) => (w.to_generic(), l.to_generic()),
            ComplexResourceRequest::AIPartner(r, d) => (r.to_generic(), d.to_generic()),
        };
        let complex_response: Result<ComplexResource, (String, GenericResource, GenericResource)> =
            Err((
                "Uncharged energy cell. Can't combine the resources".to_string(),
                r1,
                r2,
            ));

        //LOG info: "CombineResourceResponse"
        logging_wrapper::log_for_channel_with_key_info(
            state.id(),
            ActorType::Explorer,
            explorer_id.to_string(),
            EventType::MessagePlanetToExplorer,
            vec![
                (
                    "msg_enum".to_string(),
                    "CombineResourceResponse".to_string(),
                ),
                (
                    "complex_response".to_string(),
                    format!("{:?}", &complex_response),
                ),
            ],
        );
        //LOG debug: "Exit handle_combine_resource_request" + return
        logging_wrapper::log_for_channel_with_key_debug(
            state.id(),
            ActorType::SelfActor,
            "0",
            EventType::InternalPlanetAction,
            vec![
                (
                    "Debug detail".to_string(),
                    "Exit handle_combine_resource_request".to_string(),
                ),
                (
                    "return".to_string(),
                    format!(
                        "Some(PlanetToExplorer::CombineResourceResponse {{{:?}}}",
                        &complex_response
                    ),
                ),
            ],
        );
        return Some(PlanetToExplorer::CombineResourceResponse { complex_response });
    }

    let complex_response = generate_complex_resource!(planet_id, explorer_id, combinator, energy_cell, msg, {
        Diamond(c1, c2),
        Life(w, c),
        Water(h, o),
        Robot(s, l),
        Dolphin(w, l),
        AIPartner(r, d),
    });

    if complex_response.is_ok() {
        //LOG info: "Successfully combined resources"
        logging_wrapper::log_for_channel_info(
            state.id(),
            ActorType::SelfActor,
            explorer_id.to_string(),
            EventType::InternalPlanetAction,
            vec!["Successfully combined resources".to_string()],
        );
    }
    //LOG info: "CombineResourceResponse"
    logging_wrapper::log_for_channel_with_key_info(
        state.id(),
        ActorType::Explorer,
        explorer_id.to_string(),
        EventType::MessagePlanetToExplorer,
        vec![
            (
                "msg_enum".to_string(),
                "CombineResourceResponse".to_string(),
            ),
            (
                "complex_response".to_string(),
                format!("{:?}", &complex_response),
            ),
        ],
    );
    //LOG debug: "Exit handle_combine_resource_request" + return
    logging_wrapper::log_for_channel_with_key_debug(
        state.id(),
        ActorType::SelfActor,
        "0",
        EventType::InternalPlanetAction,
        vec![
            (
                "Debug detail".to_string(),
                "Exit handle_combine_resource_request".to_string(),
            ),
            (
                "return".to_string(),
                format!(
                    "Some(PlanetToExplorer::CombineResourceResponse {{{:?}}}",
                    &complex_response
                ),
            ),
        ],
    );
    Some(PlanetToExplorer::CombineResourceResponse { complex_response })
}

/// This handler returns an [PlanetToExplorer::AvailableEnergyCellResponse] message containing
/// the number of currently charged energy cells available on the planet.
/// Since the planet has only one energy cell, the value can only be 0 or 1.
///
/// # Parameters
/// - `state`: Reference to the planet state
///
/// # Returns
/// `Option<PlanetToExplorer>`
///
/// `Some(PlanetToExplorer::AvailableEnergyCellResponse)` with `available_cells` set to:
/// - **0**: if the energy cell is discharged
/// - **1**: if the energy cell is charged
///
/// # Logic
/// The handler:
/// - Initializes a counter to 0
/// - Accesses the energy cell and increments the counter if it is charged
/// - Wraps the counter inside an [PlanetToExplorer::AvailableEnergyCellResponse] message and returns it
#[allow(dead_code)]
fn handle_energy_cell_request(explorer_id: u32, state: &PlanetState) -> Option<PlanetToExplorer> {
    let planet_id = state.id();
    //LOG debug: "Called handle_energy_cell_request" + parameter
    logging_wrapper::log_for_channel_with_key_debug(
        planet_id,
        ActorType::SelfActor,
        "0",
        EventType::InternalPlanetAction,
        logging_wrapper::append_info_to_state(
            state,
            vec![(
                "Debug detail".to_string(),
                "Called handle_energy_cell_request".to_string(),
            )],
        ),
    );
    let mut available_cells = 0;
    let is_charged = state.cell(0).is_charged();
    //LOG trace: "Checking energy cell charge state: {is_charged:?}"
    logging_wrapper::log_for_channel_trace(
        planet_id,
        ActorType::SelfActor,
        explorer_id.to_string(),
        EventType::InternalPlanetAction,
        vec![format!(
            "Checking energy cell charge state: {}",
            if is_charged { "charged" } else { "discharged" }
        )],
    );
    if is_charged {
        available_cells += 1;
    }
    //LOG info: "CombineResourceResponse"
    logging_wrapper::log_for_channel_with_key_info(
        planet_id,
        ActorType::Explorer,
        explorer_id.to_string(),
        EventType::MessagePlanetToExplorer,
        vec![
            (
                "msg_enum".to_string(),
                "CombineResourceResponse".to_string(),
            ),
            (
                "available_cells".to_string(),
                format!("{:?}", &available_cells),
            ),
        ],
    );
    //LOG debug: "Exit handle_energy_cell_request" + return
    logging_wrapper::log_for_channel_with_key_debug(
        planet_id,
        ActorType::SelfActor,
        "0",
        EventType::InternalPlanetAction,
        vec![
            (
                "Debug detail".to_string(),
                "Exit handle_energy_cell_request".to_string(),
            ),
            (
                "return".to_string(),
                format!(
                    "Some(PlanetToExplorer::AvailableEnergyCellResponse{{{:?}}}",
                    &available_cells
                ),
            ),
        ],
    );
    Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells })
}

#[cfg(test)]
mod tests {
    use crate::planet_ai::{
        create_planet, handle_energy_cell_request, handle_supported_combination_request,
        handle_supported_resource_request,
    };
    use common_game::components::asteroid::Asteroid;
    use common_game::components::planet::Planet;
    use common_game::components::resource::{
        BasicResource, BasicResourceType, ComplexResource, ComplexResourceRequest,
        ComplexResourceType,
    };
    use common_game::components::sunray::Sunray;
    use common_game::protocols::messages::{
        ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
    };
    use crossbeam_channel::{Receiver, Sender, unbounded};
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex, Once};
    use std::thread;

    static INIT_LOGGER: Once = Once::new();
    pub fn init_logger() {
        INIT_LOGGER.call_once(|| {
            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Trace)
                .is_test(true)
                .init();
        });
    }

    fn planet_to_explorer_channel_creator() -> (Sender<PlanetToExplorer>, Receiver<PlanetToExplorer>)
    {
        let (planet_to_explorer_sender, planet_to_explorer_receiver): (
            Sender<PlanetToExplorer>,
            Receiver<PlanetToExplorer>,
        ) = unbounded();
        (planet_to_explorer_sender, planet_to_explorer_receiver)
    }

    fn explorer_to_planet_channels_creator()
    -> (Sender<ExplorerToPlanet>, Receiver<ExplorerToPlanet>) {
        let (explorer_to_planet_sender, explorer_to_planet_receiver): (
            Sender<ExplorerToPlanet>,
            Receiver<ExplorerToPlanet>,
        ) = unbounded();
        (explorer_to_planet_sender, explorer_to_planet_receiver)
    }

    fn orchestrator_to_planet_channels_creator()
    -> (Sender<OrchestratorToPlanet>, Receiver<OrchestratorToPlanet>) {
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver): (
            Sender<OrchestratorToPlanet>,
            Receiver<OrchestratorToPlanet>,
        ) = unbounded();
        (
            orchestrator_to_planet_sender,
            orchestrator_to_planet_receiver,
        )
    }

    fn planet_to_orchestrator_channels_creator()
    -> (Sender<PlanetToOrchestrator>, Receiver<PlanetToOrchestrator>) {
        let (planet_to_orchestrator_sender, planet_to_orchestrator_receiver): (
            Sender<PlanetToOrchestrator>,
            Receiver<PlanetToOrchestrator>,
        ) = unbounded();
        (
            planet_to_orchestrator_sender,
            planet_to_orchestrator_receiver,
        )
    }

    struct TestHarness {
        from_orch_tx: Sender<OrchestratorToPlanet>,
        to_orch_rx: Receiver<PlanetToOrchestrator>,
        from_exp_tx: Sender<ExplorerToPlanet>,
        to_exp_tx: Sender<PlanetToExplorer>,
        to_exp_rx: Receiver<PlanetToExplorer>,
        planet: Arc<Mutex<Planet>>,
        _thread: thread::JoinHandle<()>,
    }

    impl TestHarness {
        fn new() -> Self {
            init_logger();

            let (to_orch_tx, to_orch_rx) = planet_to_orchestrator_channels_creator();
            let (from_orch_tx, from_orch_rx) = orchestrator_to_planet_channels_creator();
            let (to_exp_tx, to_exp_rx) = planet_to_explorer_channel_creator();
            let (from_exp_tx, from_exp_rx) = explorer_to_planet_channels_creator();

            let planet = Arc::new(Mutex::new(create_planet(
                from_orch_rx,
                to_orch_tx,
                from_exp_rx,
                2,
            )));

            let planet_for_thread = Arc::clone(&planet);

            let handle = thread::spawn(move || {
                let _ = planet_for_thread.lock().unwrap().run();
            });

            Self {
                from_orch_tx,
                to_orch_rx,
                from_exp_tx,
                to_exp_tx,
                to_exp_rx,
                planet,
                _thread: handle,
            }
        }

        fn start(&self) {
            let _ = self.from_orch_tx.send(OrchestratorToPlanet::StartPlanetAI);
            let _ = self.to_orch_rx.recv();
        }

        fn sunray(&self) {
            let _ = self
                .from_orch_tx
                .send(OrchestratorToPlanet::Sunray(Sunray::default()));
            let _ = self.to_orch_rx.recv();
        }

        fn incoming_explorer(&self, id: u32, tx: Sender<PlanetToExplorer>) {
            let msg = OrchestratorToPlanet::IncomingExplorerRequest {
                explorer_id: id,
                new_mpsc_sender: tx,
            };
            let _ = self.from_orch_tx.send(msg);
            let _ = self.to_orch_rx.recv();
        }

        fn send_explorer(&self, msg: ExplorerToPlanet) {
            let _ = self.from_exp_tx.send(msg);
        }

        fn recv_explorer(&self) -> PlanetToExplorer {
            self.to_exp_rx.recv().unwrap()
        }
    }

    /// Assert that when the cells are not charged (which means as soon as the planet is created)
    /// the [Asteroid] destroys the [Planet].
    #[test]
    fn asteroid_with_uncharged_cell() {
        init_logger();

        // ----------------- Channels and planet creation
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) =
            orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrator_sender, planet_to_orchestrator_receiver) =
            planet_to_orchestrator_channels_creator();
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
        let _ =
            orchestrator_to_planet_sender.send(OrchestratorToPlanet::Asteroid(Asteroid::default()));
        let planet_response = planet_to_orchestrator_receiver.recv().unwrap();
        assert!(matches!(
            planet_response,
            PlanetToOrchestrator::AsteroidAck { .. }
        ));
        assert!(matches!(
            planet_response,
            PlanetToOrchestrator::AsteroidAck {
                planet_id: 2,
                rocket: None
            }
        ));
    }

    /// Assert that when the cell is charged the [Asteroid] does not destroy the [Planet].
    #[test]
    fn test_asteroid_handler_with_charged_cell() {
        init_logger();

        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) =
            orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrator_sender, planet_to_orchestrator_receiver) =
            planet_to_orchestrator_channels_creator();

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
        assert!(
            matches!(
                sunrays_planet_response,
                Ok(PlanetToOrchestrator::SunrayAck { .. })
            ),
            "Did not received a sunrays AKC"
        );
        assert!(matches!(
            sunrays_planet_response,
            Ok(PlanetToOrchestrator::SunrayAck { planet_id: 2 })
        ));

        // Send the asteroid
        let _ =
            orchestrator_to_planet_sender.send(OrchestratorToPlanet::Asteroid(Asteroid::default()));
        let planet_response = planet_to_orchestrator_receiver.recv();
        assert!(
            planet_response.is_ok(),
            "Error with the response of the planet once the Asteroid"
        );

        let planet_response_msg = planet_response.unwrap();
        assert!(
            matches!(
                planet_response_msg,
                PlanetToOrchestrator::AsteroidAck {
                    planet_id: 2,
                    rocket: _
                }
            ),
            "Planet answered with a different ID"
        );
        assert!(matches!(
            planet_response_msg,
            PlanetToOrchestrator::AsteroidAck {
                planet_id: 2,
                rocket: Some(_)
            }
        ));
        assert!(
            matches!(
                planet_response_msg,
                PlanetToOrchestrator::AsteroidAck { .. }
            ),
            "The planet did not answer back with a AsteroidAck"
        );
    }

    /// Send the rocket when the AI is not enabled
    #[test]
    fn test_rocket_with_disabled_ai() {
        init_logger();

        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) =
            orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrator_sender, planet_to_orchestrator_receiver) =
            planet_to_orchestrator_channels_creator();

        let (_, explorer_to_planet_receiver) = explorer_to_planet_channels_creator();

        let mut planet = create_planet(
            orchestrator_to_planet_receiver,
            planet_to_orchestrator_sender,
            explorer_to_planet_receiver,
            2,
        );

        let _ = thread::spawn(move || planet.run());

        // Start the AI (disabled by default)
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::StartPlanetAI);
        let _ = planet_to_orchestrator_receiver.recv();

        // Shutdown the planet AI. Note that I should not wait for the response
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::StopPlanetAI);
        let ai_shutdown_response = planet_to_orchestrator_receiver.recv();
        assert!(matches!(
            ai_shutdown_response,
            Ok(PlanetToOrchestrator::StopPlanetAIResult { .. })
        ));

        // Send the asteroid
        let _ =
            orchestrator_to_planet_sender.send(OrchestratorToPlanet::Asteroid(Asteroid::default()));
        let planet_response = planet_to_orchestrator_receiver.recv();
        assert!(matches!(
            planet_response,
            Ok(PlanetToOrchestrator::Stopped { .. })
        ));
    }

    /// Testing the start and stop of the AI.
    #[test]
    fn test_start_and_stop_planet_ai() {
        init_logger();

        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) =
            orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrator_sender, planet_to_orchestrator_receiver) =
            planet_to_orchestrator_channels_creator();

        let (_, explorer_to_planet_receiver) = explorer_to_planet_channels_creator();

        let mut planet = create_planet(
            orchestrator_to_planet_receiver,
            planet_to_orchestrator_sender,
            explorer_to_planet_receiver,
            2,
        );

        let _ = thread::spawn(move || {
            let _ = planet.run();
        });

        // Send start AI message
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::StartPlanetAI);
        let response = planet_to_orchestrator_receiver.recv();
        assert!(
            matches!(
                response,
                Ok(PlanetToOrchestrator::StartPlanetAIResult { .. })
            ),
            "Expected a StartPlanetAIResult!"
        );
        assert!(
            matches!(
                response,
                Ok(PlanetToOrchestrator::StartPlanetAIResult { planet_id: 2 })
            ),
            "Expected a StartPlanetAIResult with id = 2!"
        );

        // Send stop AI message
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::StopPlanetAI);
        let response = planet_to_orchestrator_receiver.recv();
        assert!(
            matches!(
                response,
                Ok(PlanetToOrchestrator::StopPlanetAIResult { .. })
            ),
            "Expected a StopPlanetAIResult"
        );
        assert!(
            matches!(
                response,
                Ok(PlanetToOrchestrator::StopPlanetAIResult { planet_id: 2 })
            ),
            "Expected a StopPlanetAIResult with id = 2"
        );

        // Again, send startAI
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::StartPlanetAI);
        let response = planet_to_orchestrator_receiver.recv();
        assert!(
            matches!(
                response,
                Ok(PlanetToOrchestrator::StartPlanetAIResult { .. })
            ),
            "Expected a StartPlanetAIResult!"
        );
        assert!(
            matches!(
                response,
                Ok(PlanetToOrchestrator::StartPlanetAIResult { planet_id: 2 })
            ),
            "Expected a StartPlanetAIResult with id = 2!"
        );
    }

    #[test]
    fn test_unit_handle_supported_resource_request() {
        let h = TestHarness::new();
        let planet = h.planet.lock().unwrap();

        let result = handle_supported_resource_request(2, 1, planet.generator());

        assert!(result.is_some());

        if let Some(PlanetToExplorer::SupportedResourceResponse { resource_list }) = result {
            let result_set: HashSet<BasicResourceType> = resource_list.into_iter().collect();
            let expected_set: HashSet<BasicResourceType> =
                vec![BasicResourceType::Carbon].into_iter().collect();
            assert_eq!(result_set, expected_set);
        } else {
            panic!("Expected SupportedResourceResponse variant");
        }
    }

    #[test]
    fn test_unit_handle_supported_combination_request() {
        let h = TestHarness::new();
        let planet = h.planet.lock().unwrap();

        let result = handle_supported_combination_request(2, 1, planet.combinator());

        assert!(result.is_some());

        if let Some(PlanetToExplorer::SupportedCombinationResponse { combination_list }) = result {
            let result_set: HashSet<ComplexResourceType> = combination_list.into_iter().collect();
            let expected_set: HashSet<ComplexResourceType> = vec![
                ComplexResourceType::Diamond,
                ComplexResourceType::Life,
                ComplexResourceType::AIPartner,
                ComplexResourceType::Dolphin,
                ComplexResourceType::Robot,
                ComplexResourceType::Water,
            ]
            .into_iter()
            .collect();
            assert_eq!(result_set, expected_set);
        } else {
            panic!("Expected SupportedCombinationResponse variant");
        }
    }

    #[test]
    fn test_integration_generate_carbon_with_energy() {
        let h = TestHarness::new();

        h.start();
        h.sunray();
        h.incoming_explorer(1, h.to_exp_tx.clone());

        h.send_explorer(ExplorerToPlanet::GenerateResourceRequest {
            explorer_id: 1,
            resource: BasicResourceType::Carbon,
        });

        match h.recv_explorer() {
            PlanetToExplorer::GenerateResourceResponse { resource } => {
                assert!(matches!(resource, Some(BasicResource::Carbon(_))));
            }
            _ => panic!("Expected GenerateResourceResponse"),
        }
    }

    #[test]
    fn test_integration_generate_unsupported_resource() {
        let h = TestHarness::new();

        h.start();
        h.sunray();
        h.incoming_explorer(1, h.to_exp_tx.clone());

        h.send_explorer(ExplorerToPlanet::GenerateResourceRequest {
            explorer_id: 1,
            resource: BasicResourceType::Oxygen,
        });

        match h.recv_explorer() {
            PlanetToExplorer::GenerateResourceResponse { resource } => assert!(resource.is_none()),
            _ => panic!("Expected GenerateResourceResponse"),
        }
    }

    #[test]
    fn test_integration_generate_carbon_without_energy() {
        let h = TestHarness::new();

        h.start();
        h.incoming_explorer(1, h.to_exp_tx.clone());

        h.send_explorer(ExplorerToPlanet::GenerateResourceRequest {
            explorer_id: 1,
            resource: BasicResourceType::Carbon,
        });

        match h.recv_explorer() {
            PlanetToExplorer::GenerateResourceResponse { resource } => assert!(resource.is_none()),
            _ => panic!("Expected GenerateResourceResponse"),
        }
    }

    #[test]
    fn test_integration_combine_carbon_with_energy() {
        let h = TestHarness::new();
        h.start();
        h.incoming_explorer(1, h.to_exp_tx.clone());

        let mut bag = Vec::new();

        for _ in 0..2 {
            h.sunray();
            h.send_explorer(ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: 1,
                resource: BasicResourceType::Carbon,
            });
            if let PlanetToExplorer::GenerateResourceResponse {
                resource: Some(BasicResource::Carbon(c)),
            } = h.recv_explorer()
            {
                bag.push(c);
            } else {
                panic!("Expected Carbon");
            }
        }

        h.sunray();

        h.send_explorer(ExplorerToPlanet::CombineResourceRequest {
            explorer_id: 1,
            msg: ComplexResourceRequest::Diamond(bag.pop().unwrap(), bag.pop().unwrap()),
        });

        match h.recv_explorer() {
            PlanetToExplorer::CombineResourceResponse {
                complex_response: Ok(ComplexResource::Diamond(_)),
            } => {}
            _ => panic!("Expected Ok(Diamond)"),
        }
    }

    #[test]
    fn test_integration_combine_carbon_without_energy() {
        let h = TestHarness::new();
        h.start();
        h.incoming_explorer(1, h.to_exp_tx.clone());

        let mut bag = Vec::new();

        for _ in 0..2 {
            h.sunray();
            h.send_explorer(ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: 1,
                resource: BasicResourceType::Carbon,
            });
            if let PlanetToExplorer::GenerateResourceResponse {
                resource: Some(BasicResource::Carbon(c)),
            } = h.recv_explorer()
            {
                bag.push(c);
            } else {
                panic!("Expected Carbon");
            }
        }

        h.send_explorer(ExplorerToPlanet::CombineResourceRequest {
            explorer_id: 1,
            msg: ComplexResourceRequest::Diamond(bag.pop().unwrap(), bag.pop().unwrap()),
        });

        match h.recv_explorer() {
            PlanetToExplorer::CombineResourceResponse { complex_response } => {
                assert!(
                    complex_response.is_err(),
                    "Expected Err(), got {:?}",
                    complex_response
                );
            }
            _ => panic!("Expected CombineResourceResponse"),
        }
    }

    #[test]
    fn test_integration_handle_orchestrator_msg_sunray_and_handle_energy_cell_request_charge() {
        let h = TestHarness::new();

        h.start();
        h.sunray();
        h.incoming_explorer(1, h.to_exp_tx.clone());

        h.from_exp_tx
            .send(ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: 1 })
            .unwrap();

        let response = h.to_exp_rx.recv().unwrap();

        match response {
            PlanetToExplorer::AvailableEnergyCellResponse { available_cells } => {
                assert_eq!(available_cells, 1);
            }
            _ => panic!("Expected AvailableEnergyCellResponse"),
        }
    }

    #[test]
    fn test_unit_handle_energy_cell_request_discharge() {
        let h = TestHarness::new();
        let planet = h.planet.lock().unwrap();

        let result = handle_energy_cell_request(1, planet.state());

        assert!(result.is_some());

        if let Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells }) = result {
            let expected_usize = 0;
            assert_eq!(available_cells, expected_usize);
        } else {
            panic!("Expected SupportedCombinationResponse variant");
        }
    }

    #[test]
    fn test_internal_state_request() {
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) =
            orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrator_sender, planet_to_orchestrator_receiver) =
            planet_to_orchestrator_channels_creator();
        let (_, explorer_to_planet_receiver) = explorer_to_planet_channels_creator();

        let test_planet_id = 10;

        let mut planet = create_planet(
            orchestrator_to_planet_receiver,
            planet_to_orchestrator_sender,
            explorer_to_planet_receiver,
            test_planet_id,
        );

        let planet_thread = thread::spawn(move || {
            let _ = planet.run();
        });

        orchestrator_to_planet_sender
            .send(OrchestratorToPlanet::StartPlanetAI)
            .expect("Failed to send StartPlanetAI");

        let _ = planet_to_orchestrator_receiver.recv();

        orchestrator_to_planet_sender
            .send(OrchestratorToPlanet::InternalStateRequest)
            .expect("Failed to send InternalStateRequest");

        let response = planet_to_orchestrator_receiver.recv();

        match response {
            Ok(PlanetToOrchestrator::InternalStateResponse {
                planet_id,
                planet_state,
            }) => {
                assert_eq!(
                    planet_id, test_planet_id,
                    "Returned planet ID does not match"
                );

                assert_eq!(
                    planet_state.energy_cells.len(),
                    1,
                    "Planet C should have 1 energy cell"
                );
            }
            Ok(_) => panic!("Received unexpected message type"),
            Err(e) => panic!("Failed to receive response: {:?}", e),
        }

        orchestrator_to_planet_sender
            .send(OrchestratorToPlanet::KillPlanet)
            .expect("Failed to send KillPlanet");

        let _ = planet_thread.join();
    }
}
