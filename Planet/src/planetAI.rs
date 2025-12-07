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
use common_game::protocols::messages::*;

use paste::paste;

trait PlanetDefinition {
    fn get_name() -> &'static str;
    fn get_type() -> &'static PlanetType;
}


struct CargonautsPlanet {
    ai_is_active: bool
}

/// Function that create a Planet with specific arguments TODO
pub fn create_planet(
    orch_channels: (Receiver<OrchestratorToPlanet>, Sender<PlanetToOrchestrator>),
    explorer_channels: Receiver<ExplorerToPlanet>,
    ai: Box<dyn PlanetAI>
) -> Planet {
    let planet = Planet::new(
        2,
        PlanetType::C,
        ai,
        vec![BasicResourceType::Carbon],
        vec![ComplexResourceType::Diamond, ComplexResourceType::Life],
        orch_channels,
        explorer_channels
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
        Self {
            ai_is_active: true
        }
    }
}

impl PlanetAI for CargonautsPlanet {
    fn handle_orchestrator_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: OrchestratorToPlanet,
    ) -> Option<PlanetToOrchestrator> {

        match msg {
            // Charge single cell at vector (position 0 because the planet is of Type C
            OrchestratorToPlanet::Sunray(sunray) => {
                let cell = state.cell_mut(0);
                cell.charge(sunray);

                Some(PlanetToOrchestrator::SunrayAck {
                    planet_id: state.id(),
                })

            }

            // Use the method to be implemented later
            OrchestratorToPlanet::Asteroid(_) => {
                let maybe_rocket = self.handle_asteroid(state, generator, combinator);

                Some(PlanetToOrchestrator::AsteroidAck {
                    planet_id: state.id(),
                    rocket: maybe_rocket,
                })
            }

            //same here and for stop planetAi
            OrchestratorToPlanet::StartPlanetAI => {
                self.start(state);

                Some(PlanetToOrchestrator::StartPlanetAIResult {
                    planet_id: state.id(),
                })
            }

            OrchestratorToPlanet::StopPlanetAI => {
                self.stop(state);

                Some(PlanetToOrchestrator::StopPlanetAIResult {
                    planet_id: state.id(),
                })
            }

            OrchestratorToPlanet::InternalStateRequest=> {
                todo!(
                    "Waiting for upstream fix: PlanetState allows no cloning nor manual construction"
                );
            }

            OrchestratorToPlanet::IncomingExplorerRequest {
                explorer_id: _,
                new_mpsc_sender: _
            } => {
                todo!()
            }

            OrchestratorToPlanet::OutgoingExplorerRequest {explorer_id: _} => {
                todo!();
            },
            OrchestratorToPlanet::KillPlanet => {
                todo!()
            }
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
                println!("SupportedResourceRequest received from explorer[{}]", explorer_id);
                handle_supported_resource_request(generator)
            },
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                println!("SupportedCombinationRequest received from explorer[{}]", explorer_id);
                handle_supported_combination_request(combinator)
            },
            ExplorerToPlanet::GenerateResourceRequest { explorer_id, resource } => {
                println!("GenerateResourceRequest received from explorer[{}]. Ask for generate {:?}", explorer_id, resource);
                handle_generate_resource_request(state, generator, resource)
            },
            ExplorerToPlanet::CombineResourceRequest { explorer_id, msg } => {
                println!("CombineResourceRequest received from explorer[{}]. Ask for craft {:?}", explorer_id, msg);
                handle_combine_resource_request(state, combinator, msg)
            },
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id } => {
                println!("AvailableEnergyCellRequest received from explorer[{}]", explorer_id);
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

        if !self.ai_is_active || !state.can_have_rocket(){
            return None;
        }

        // At this point the Rocket can be built. Check if there already
        // is a rocket ready to be used
        if state.has_rocket() {
            let rocket = state.take_rocket().unwrap();
            Some(rocket)
        } else {
            // The rocket is not available, check if it still can be created with the use of an
            // EnergyCell.
            let charged_cell_position = state.cells_iter().position( |cell_ref| cell_ref.is_charged() );
            if let Some(charged_cell_position_result) = charged_cell_position {
                // Create the rocket and return it
                let created_rocket_result = state.build_rocket( charged_cell_position_result );
                if let Ok(non_err_msg) = created_rocket_result {
                    return state.take_rocket();
                }
            }
            // Rocket can not be built
            None
        }
    }


    /// This method will be invoked when a [OrchestratorToPlanet::StartPlanetAI]
    /// is received, but **only if** the planet is currently in a *stopped* state.
    ///
    /// Start messages received when planet is already running are **ignored**.
    fn start(&mut self, state: &PlanetState) {
        self.ai_is_active = true;
    }


    /// This method will be invoked when a [OrchestratorToPlanet::StopPlanetAI]
    /// is received, but **only if** the planet is currently in a *running* state.
    ///
    fn stop(&mut self, state: &PlanetState) {
        self.ai_is_active = false;
    }
}

// === OrchestratorToPlanet Handler ================================================================


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
                    Err(e) => panic!("{:?}", e) //TODO right?
                }
            },
            _ => panic!("Unexpected resource type") //TODO right?
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


    use crossbeam_channel::unbounded;
    use std::sync::{Arc, Mutex};
    use std::collections::HashSet;
    use std::thread;
    use common_game::components::asteroid::Asteroid;
    use common_game::components::sunray::Sunray;
    use common_game::components::resource::{BasicResourceType, ComplexResourceType};
    use common_game::components::planet::{Planet, PlanetAI, PlanetType};
    use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};
    use crossbeam_channel::{unbounded, Receiver, Sender};
    use crate::planetAI::{handle_energy_cell_request, handle_supported_combination_request, handle_supported_resource_request, CargonautsPlanet, create_planet};


    fn planet_to_explorer_channel_creator() -> (crossbeam_channel::Sender<PlanetToExplorer>, crossbeam_channel::Receiver<PlanetToExplorer>) {
        let (planet_to_explorer_sender, planet_to_explorer_receiver): (crossbeam_channel::Sender<PlanetToExplorer>, crossbeam_channel::Receiver<PlanetToExplorer>) = crossbeam_channel::unbounded();
        (planet_to_explorer_sender, planet_to_explorer_receiver)
    }


    fn explorer_to_planet_channels_creator() -> (crossbeam_channel::Sender<ExplorerToPlanet>, crossbeam_channel::Receiver<ExplorerToPlanet>) {
        let (explorer_to_planet_sender, explorer_to_planet_receiver): (crossbeam_channel::Sender<ExplorerToPlanet>, crossbeam_channel::Receiver<ExplorerToPlanet>) = crossbeam_channel::unbounded();
        (explorer_to_planet_sender, explorer_to_planet_receiver)
    }

    fn orchestrator_to_planet_channels_creator() -> (crossbeam_channel::Sender<OrchestratorToPlanet>, crossbeam_channel::Receiver<OrchestratorToPlanet>) {
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver): (crossbeam_channel::Sender<OrchestratorToPlanet>, crossbeam_channel::Receiver<OrchestratorToPlanet>) = crossbeam_channel::unbounded();
        (orchestrator_to_planet_sender, orchestrator_to_planet_receiver)
    }


    fn planet_to_orchestrator_channels_creator() -> (crossbeam_channel::Sender<PlanetToOrchestrator>, crossbeam_channel::Receiver<PlanetToOrchestrator>) {
        let (planet_to_orchestrato_sender, planet_to_orchestrator_receiver): (crossbeam_channel::Sender<PlanetToOrchestrator>, crossbeam_channel::Receiver<PlanetToOrchestrator>) = crossbeam_channel::unbounded();
        (planet_to_orchestrato_sender, planet_to_orchestrator_receiver)
    }

    fn create_planet_t(
        (planet_to_orchestrator_sender, orchestrator_to_planet_receiver): (crossbeam_channel::Sender<PlanetToOrchestrator>, crossbeam_channel::Receiver<OrchestratorToPlanet>),
        explorer_to_planet_receiver: crossbeam_channel::Receiver<ExplorerToPlanet>,
        ai: Box<dyn PlanetAI>
    ) -> Planet {
        let planet = Planet::new(
            2,
            PlanetType::C,
            ai,
            vec![BasicResourceType::Silicon],
            vec![ComplexResourceType::Diamond, ComplexResourceType::AIPartner],
            (orchestrator_to_planet_receiver, planet_to_orchestrator_sender),
            explorer_to_planet_receiver
        );
        assert!(planet.is_ok(), "Planet creatrion error!");
        planet.unwrap()
    }

    /// Assert that when the cells is not charged (which means as soon as the planet is created)
    /// the [Asteroid] destroys the [Planet].
    #[test]
    fn asteroid_with_uncharged_cell() {

        // ----------------- Channels and planet cration
        let toy_struct = CargonautsPlanet::default();
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) = orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrator_sender, planet_to_orchestrator_receiver) = planet_to_orchestrator_channels_creator();
        let (_, explorer_to_planet_receiver) = explorer_to_planet_channels_creator();
        let (planet_to_explorer_sender, _) = planet_to_explorer_channel_creator();
        let mut planet = create_planet_t(
            (planet_to_orchestrator_sender, orchestrator_to_planet_receiver, ),
            explorer_to_planet_receiver,
            Box::from(toy_struct)
        );

        // ----------------- Spawn the thread:
        let _ = thread::spawn(move || {
            let _ = planet.run();
        });

        // ----------------- Make the planet start
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::StartPlanetAI);


        // ----------------- Send an asteroid
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::Asteroid(Asteroid::default()));
        let planet_response = planet_to_orchestrator_receiver.recv().unwrap();
        assert!(matches!( planet_response, PlanetToOrchestrator::AsteroidAck { .. } ));
        assert!(matches!( planet_response, PlanetToOrchestrator::AsteroidAck { planet_id: 2, rocket: None }));
    }


    /// Assert that when the cell is charged the [Asteroid] does not destroy the [Planet].
    #[test]
    fn test_asteroid_handler_with_charged_cell() {
        let toy_struct = CargonautsPlanet::default();
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) = orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrator_sender, planet_to_orchestrator_receiver) = planet_to_orchestrator_channels_creator();

        let (_, explorer_to_planet_receiver) = explorer_to_planet_channels_creator();
        let (planet_to_explorer_sender, _) = planet_to_explorer_channel_creator();


        let mut planet = create_planet_t(
            (planet_to_orchestrator_sender, orchestrator_to_planet_receiver),
            explorer_to_planet_receiver,
            Box::from(toy_struct)
        );

        // Spawn the thread:
        let _ = thread::spawn(move || {
            let _ = planet.run();
        });

        // Make the PlanetAI start
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::StartPlanetAI);


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

        let toy_struct = CargonautsPlanet::default();
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) = orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrato_sender, planet_to_orchestrator_receiver) = planet_to_orchestrator_channels_creator();

        let (_, explorer_to_planet_receiver) = explorer_to_planet_channels_creator();
        let (planet_to_explorer_sender, _) = planet_to_explorer_channel_creator();


        let mut planet = create_planet_t(
            (planet_to_orchestrato_sender, orchestrator_to_planet_receiver),
            explorer_to_planet_receiver,
            Box::from(toy_struct)
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

        let toy_struct = CargonautsPlanet::default();
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) = orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrator_sender, planet_to_orchestrator_receiver) = planet_to_orchestrator_channels_creator();

        let (_, explorer_to_planet_receiver) = explorer_to_planet_channels_creator();
        let (planet_to_explorer_sender, _) = planet_to_explorer_channel_creator();


        let mut planet = create_planet_t(
            (planet_to_orchestrator_sender, orchestrator_to_planet_receiver),
            explorer_to_planet_receiver,
            Box::from(toy_struct)
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
        let (to_orchestrator_tx, _to_orchestrator_rx) = unbounded(); // Planet -> Orchestrator
        let (_from_orchestrator_tx, from_orchestrator_rx) = unbounded(); // Orchestrator -> Planet
        let (to_explorer_tx, _to_explorer_rx) = explorer_to_planet_channels_creator(); // Planet -> Explorer
        let (_from_explorer_tx, from_explorer_rx) = unbounded(); // Explorer -> Planet

        let planet = create_planet(
            (from_orchestrator_rx, to_orchestrator_tx),
            from_explorer_rx,
            Box::from(CargonautsPlanet::default())
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
        let (to_orchestrator_tx, _to_orchestrator_rx) = unbounded(); // Planet -> Orchestrator
        let (_from_orchestrator_tx, from_orchestrator_rx) = unbounded(); // Orchestrator -> Planet
        let (to_explorer_tx, _to_explorer_rx) = explorer_to_planet_channels_creator(); // Planet -> Explorer
        let (_from_explorer_tx, from_explorer_rx) = unbounded(); // Explorer -> Planet

        let planet = create_planet(
            (from_orchestrator_rx, to_orchestrator_tx),
            from_explorer_rx,
            Box::from(CargonautsPlanet::default())
        );

        let result = handle_supported_combination_request(planet.combinator());

        assert!(result.is_some());

        if let Some(PlanetToExplorer::SupportedCombinationResponse { combination_list }) = result {
            let result_set: HashSet<ComplexResourceType> = combination_list.into_iter().collect();
            let expected_set: HashSet<ComplexResourceType> = vec![ComplexResourceType::Diamond, ComplexResourceType::Life].into_iter().collect();
            assert_eq!(result_set, expected_set);
        } else {
            panic!("Expected SupportedCombinationResponse variant");
        }
    }

    #[test]
    fn test_integration_handle_orchestrator_msg_sunray_and_handle_energy_cell_request_charge() {
        let (to_orchestrator_tx, _to_orchestrator_rx) = unbounded();
        let (from_orchestrator_tx, from_orchestrator_rx) = unbounded();
        let (to_explorer_tx, _to_explorer_rx) = explorer_to_planet_channels_creator();
        let (_from_explorer_tx, from_explorer_rx) = unbounded();

        let planet = Arc::new(Mutex::new(create_planet(
            (from_orchestrator_rx, to_orchestrator_tx),
            from_explorer_rx,
            Box::from(CargonautsPlanet::default())
        )));

        let planet_for_thread = Arc::clone(&planet);

        let thread_var = thread::spawn(move || {
            planet_for_thread.lock().unwrap().run();
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
        let (to_orchestrator_tx, _to_orchestrator_rx) = unbounded(); // Planet -> Orchestrator
        let (_from_orchestrator_tx, from_orchestrator_rx) = unbounded(); // Orchestrator -> Planet
        let (to_explorer_tx, _to_explorer_rx) = explorer_to_planet_channels_creator(); // Planet -> Explorer
        let (_from_explorer_tx, from_explorer_rx) = unbounded(); // Explorer -> Planet

        let planet = create_planet(
            (from_orchestrator_rx, to_orchestrator_tx),
            from_explorer_rx,
            Box::from(CargonautsPlanet::default())
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