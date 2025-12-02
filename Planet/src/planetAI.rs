use std::os::linux::raw::stat;
use std::time::SystemTime;
use common_game::components::planet::*;
use common_game::components::resource::ComplexResourceRequest;
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::messages::*;
use common_game::components::resource::{Combinator, Generator};



struct CargonautsPlanet {
    ai_is_active: bool
}

impl Default for CargonautsPlanet {
    fn default() -> Self {
        Self {
            ai_is_active: true
        }
    }
}

impl PlanetAI for CargonautsPlanet  {
    fn handle_orchestrator_msg(
        &mut self, state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: OrchestratorToPlanet
    ) -> Option<PlanetToOrchestrator> {
        match msg {
            OrchestratorToPlanet::Sunray(sunray) => {
                let cell = state.cell_mut(0);
                cell.charge(sunray);

                Some(PlanetToOrchestrator::SunrayAck {
                    planet_id: state.id(),
                    timestamp: SystemTime::now(),
                })
            },
            OrchestratorToPlanet::Asteroid(_) => {
                let result = self.handle_asteroid( state, generator, combinator );
                Some(PlanetToOrchestrator::AsteroidAck { planet_id: state.id(), rocket: result})
            }, //Handled in start method
            //OrchestratorToPlanet::StartPlanetAI(_) => {}
            OrchestratorToPlanet::StopPlanetAI(_) => None, //Handled in start method
            //OrchestratorToPlanet::ManualStopPlanetAI(_) => {}
            //OrchestratorToPlanet::ManualStartPlanetAI(_) => {}
            OrchestratorToPlanet::InternalStateRequest(msg) => {
                handle_internal_state_request_orch(state, msg)
            }
            _ => None //TODO Remove after is defined where to manage StartPlanetAI, ManualStopPlanetAI, ManualStartPlanetAI
        }
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
                handle_supported_resource_request(state, explorer_id)
            },
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                handle_supported_combination_request(state, explorer_id)
            },
            ExplorerToPlanet::CombineResourceRequest { explorer_id, msg } => {
                handle_combine_resource_request(state, explorer_id, msg)
            },
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id } => {
                handle_energy_cell_request(state, explorer_id)
            },
            ExplorerToPlanet::InternalStateRequest { explorer_id } => {
                handle_internal_state_request(state, explorer_id)
            }
            _ => todo!()
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

    fn start(&mut self, state: &PlanetState) {
        self.ai_is_active = true;
    }

    fn stop(&mut self, state: &PlanetState) {
        todo!()
    }
}

// === OrchestratorToPlanet Handler ================================================================

fn handle_sunray(
    state: &mut PlanetState,
    ray: Sunray,
) -> Option<PlanetToOrchestrator> {
    todo!()
}

fn handle_internal_state_request_orch(
    state: &mut PlanetState,
    msg: InternalStateRequestMsg,
) -> Option<PlanetToOrchestrator> {
    todo!()
}

// === ExplorerToPlanet Handler ====================================================================
fn handle_supported_resource_request(
    state: &mut PlanetState,
    explorer_id: u32,
) -> Option<PlanetToExplorer> {
    todo!()
}

fn handle_supported_combination_request(
    state: &mut PlanetState,
    explorer_id: u32,
) -> Option<PlanetToExplorer> {
    todo!()
}

fn handle_generate_resource_request(
    state: &mut PlanetState,
    explorer_id: u32,
    msg: GenerateResourceRequest,
) -> Option<PlanetToExplorer> {
    todo!()
}

fn handle_combine_resource_request(
    state: &mut PlanetState,
    explorer_id: u32,
    msg: ComplexResourceRequest,
) -> Option<PlanetToExplorer> {
    todo!()
}

fn handle_energy_cell_request(
    state: &mut PlanetState,
    explorer_id: u32,
) -> Option<PlanetToExplorer> {
    todo!()
}

fn handle_internal_state_request(
    state: &mut PlanetState,
    explorer_id: u32,
) -> Option<PlanetToExplorer> {
    todo!()
}


// ---------------- Asteroid Handler Tests -----------
#[cfg(test)]
mod asteroid_handler_test {
    use std::sync::mpsc;
    use common_game::components::planet::{Planet, PlanetAI, PlanetType};
    use common_game::components::resource::{BasicResourceType, ComplexResourceType};
    use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator, StartPlanetAiMsg};
    use crate::planetAI::CargonautsPlanet;
    use std::thread;
    use common_game::components::asteroid::Asteroid;
    use common_game::components::sunray::Sunray;


    fn planet_to_explorer_channel_creator() ->  (mpsc::Sender<PlanetToExplorer>, mpsc::Receiver<PlanetToExplorer>) {
        let (planet_to_explorer_sender, planet_to_explorer_receiver) : (mpsc::Sender<PlanetToExplorer>, mpsc::Receiver<PlanetToExplorer>) = mpsc::channel();
        (planet_to_explorer_sender, planet_to_explorer_receiver)
    }

    fn explorer_to_planet_channels_creator() -> (mpsc::Sender<ExplorerToPlanet>, mpsc::Receiver<ExplorerToPlanet>) {
        let (explorer_to_planet_sender, explorer_to_planet_receiver) : (mpsc::Sender<ExplorerToPlanet>, mpsc::Receiver<ExplorerToPlanet>) = mpsc::channel();
        (explorer_to_planet_sender, explorer_to_planet_receiver)
    }

    fn orchestrator_to_planet_channels_creator() -> (mpsc::Sender<OrchestratorToPlanet>, mpsc::Receiver<OrchestratorToPlanet>) {
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) : (mpsc::Sender<OrchestratorToPlanet>, mpsc::Receiver<OrchestratorToPlanet>) = mpsc::channel();
        (orchestrator_to_planet_sender, orchestrator_to_planet_receiver)
    }

    fn planet_to_orchestrator_channels_crator() -> (mpsc::Sender<PlanetToOrchestrator>, mpsc::Receiver<PlanetToOrchestrator>) {
        let (planet_to_orchestrato_sender, planet_to_orchestrator_receiver) : (mpsc::Sender<PlanetToOrchestrator>, mpsc::Receiver<PlanetToOrchestrator>) = mpsc::channel();
        (planet_to_orchestrato_sender, planet_to_orchestrator_receiver)
    }

    fn create_planet<T: PlanetAI>(
        (planet_to_orchestrator_sender, orchestrator_to_planet_receiver) : (mpsc::Sender<PlanetToOrchestrator>, mpsc::Receiver<OrchestratorToPlanet>),
        (planet_to_explorer_sender, explorer_to_planet_receiver) : (mpsc::Sender<PlanetToExplorer>, mpsc::Receiver<ExplorerToPlanet>),
        ai: T
    ) -> Planet<T> {
        let planet = Planet::new(
            2,
            PlanetType::C,
            ai,
            vec![BasicResourceType::Silicon],
            vec![ComplexResourceType::Diamond, ComplexResourceType::AIPartner],
            ( orchestrator_to_planet_receiver, planet_to_orchestrator_sender ),
            (explorer_to_planet_receiver, planet_to_explorer_sender)
        );
        assert!(planet.is_ok(), "Planet creatrion error!");
        planet.unwrap()
    }

    #[test]
    fn asteroid_with_uncharged_cell() {
        let toy_struct = CargonautsPlanet::default();
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver)  = orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrato_sender, planet_to_orchestrator_receiver) = planet_to_orchestrator_channels_crator();

        let (explorer_to_planet_sender, explorer_to_planet_receiver)  = explorer_to_planet_channels_creator();
        let (planet_to_explorer_sender, planet_to_explorer_receiver)  = planet_to_explorer_channel_creator();


        let mut planet = create_planet(
            (planet_to_orchestrato_sender, orchestrator_to_planet_receiver),
            (planet_to_explorer_sender, explorer_to_planet_receiver),
            toy_struct
        );

        // Spawn the thread:
        let therad_var = thread::spawn( move || {
            planet.run();
        });

        // Make the planet start
        let _ = orchestrator_to_planet_sender.send( OrchestratorToPlanet::StartPlanetAI( StartPlanetAiMsg ) );

        // Send an asteroid
        let _ = orchestrator_to_planet_sender.send( OrchestratorToPlanet::Asteroid( Asteroid::default()) );
        let planet_response = planet_to_orchestrator_receiver.recv().unwrap();
        assert!(matches!( planet_response, PlanetToOrchestrator::AsteroidAck { .. } ));
        assert!(matches!( planet_response, PlanetToOrchestrator::AsteroidAck { planet_id: 2, rocket: None }));
    }


    #[test]
    fn test_asteroid_handler_with_charged_cell() {
        let toy_struct = CargonautsPlanet::default();
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver)  = orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrato_sender, planet_to_orchestrator_receiver) = planet_to_orchestrator_channels_crator();

        let (explorer_to_planet_sender, explorer_to_planet_receiver)  = explorer_to_planet_channels_creator();
        let (planet_to_explorer_sender, planet_to_explorer_receiver)  = planet_to_explorer_channel_creator();


        let mut planet = create_planet(
            (planet_to_orchestrato_sender, orchestrator_to_planet_receiver),
            (planet_to_explorer_sender, explorer_to_planet_receiver),
            toy_struct
        );

        // Spawn the thread:
        let therad_var = thread::spawn( move || {
            planet.run();
        });

        // Make the planet start
        let _ = orchestrator_to_planet_sender.send( OrchestratorToPlanet::StartPlanetAI( StartPlanetAiMsg ) );


        // Send sunrays
        let _ = orchestrator_to_planet_sender.send( OrchestratorToPlanet::Sunray( Sunray::default() ) );
        let sunrays_planet_response = planet_to_orchestrator_receiver.recv();
        assert!( matches!( sunrays_planet_response.unwrap(), PlanetToOrchestrator::SunrayAck { .. } ), "Did not received a sunrays AKC" );

        // Send the asteroid
        let _ = orchestrator_to_planet_sender.send( OrchestratorToPlanet::Asteroid( Asteroid::default() ) );
        let planet_response = planet_to_orchestrator_receiver.recv();
        assert!(planet_response.is_ok(), "Error with the response of the planet once the Asteroid");

        if let Ok( planet_response_msg ) = planet_response {
            assert!( matches!( planet_response_msg, PlanetToOrchestrator::AsteroidAck { planet_id: 2,  rocket: _ }), "Planet answered with a different ID");
            assert!( matches!( planet_response_msg, PlanetToOrchestrator::AsteroidAck { planet_id: 2,  rocket: Some( _ ) }));
            assert!( matches!(planet_response_msg, PlanetToOrchestrator::AsteroidAck { .. } ), "The planet did not answer back with a AsteroidAck");
        }
    }




}