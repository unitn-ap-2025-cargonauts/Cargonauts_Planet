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
            OrchestratorToPlanet::Sunray(ray) => {
                handle_sunray(state, ray)
            },
            OrchestratorToPlanet::Asteroid(_) => None, //Handled in start method
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

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator
    ) -> Option<Rocket> {

        if !self.ai_is_active {
            return None;
        }

        if let Some(rocket) = state.take_rocket() {
            println!("An asteroid ☄️ is coming... Luckly we have a rocket ready to be launched 🚀!");
            Some( rocket )
        } else {
            println!("An asteroid ☄️ is coming... Unluckly we do not have any rocker to launch!");
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


// ---------------- Tests
#[cfg(test)]
mod test {
    use std::sync::mpsc;
    use common_game::components::planet::{Planet, PlanetType};
    use common_game::components::resource::{BasicResourceType, ComplexResourceType};
    use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};
    use crate::planetAI::CargonautsPlanet;
    use std::thread;
    use common_game::components::asteroid::Asteroid;

    #[test]
    fn test_rocket_handler_with_no_rocket() {

        let toy_struct = CargonautsPlanet::default();
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) : (mpsc::Sender<OrchestratorToPlanet>, mpsc::Receiver<OrchestratorToPlanet>) = mpsc::channel();
        let (planet_to_orchestrato_sender, planet_to_orchestrator_receiver) : (mpsc::Sender<PlanetToOrchestrator>, mpsc::Receiver<PlanetToOrchestrator>) = mpsc::channel();

        let (explorer_to_planet_sender, explorer_to_planet_receiver) : (mpsc::Sender<ExplorerToPlanet>, mpsc::Receiver<ExplorerToPlanet>) = mpsc::channel();
        let (planet_to_explorer_sender, planet_to_explorer_receiver) : (mpsc::Sender<PlanetToExplorer>, mpsc::Receiver<PlanetToExplorer>) = mpsc::channel();


        let planet = Planet::new(
            2,
            PlanetType::C,
            toy_struct,
            vec![BasicResourceType::Silicon],
            vec![ComplexResourceType::Diamond, ComplexResourceType::AIPartner],
            ( orchestrator_to_planet_receiver, planet_to_orchestrato_sender ),
            (explorer_to_planet_receiver, planet_to_explorer_sender)
        );


        assert!(planet.is_ok(), "Error on creating the planet");

        let mut unwrapped_planet = planet.unwrap();

        let thread_plane = thread::spawn(move|| {
            unwrapped_planet.run();
        });


        let asteroid_to_be_send = Asteroid::new();
        let sent_message_status = orchestrator_to_planet_sender.send( OrchestratorToPlanet::Asteroid( asteroid_to_be_send  ) );
        assert!(sent_message_status.is_ok());

        let result_message = planet_to_orchestrator_receiver.recv().unwrap();
        assert!( matches!(result_message, PlanetToOrchestrator::AsteroidAck {planet_id : 2, rocket:  None }) , "Expected rocket not found in the response" )
    }

    #[test]
    fn test_rocker_handler_with_rocker() {
        let toy_struct = CargonautsPlanet::default();
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) : (mpsc::Sender<OrchestratorToPlanet>, mpsc::Receiver<OrchestratorToPlanet>) = mpsc::channel();
        let (planet_to_orchestrato_sender, planet_to_orchestrator_receiver) : (mpsc::Sender<PlanetToOrchestrator>, mpsc::Receiver<PlanetToOrchestrator>) = mpsc::channel();
        let (explorer_to_planet_sender, explorer_to_planet_receiver) : (mpsc::Sender<ExplorerToPlanet>, mpsc::Receiver<ExplorerToPlanet>) = mpsc::channel();
        let (planet_to_explorer_sender, planet_to_explorer_receiver) : (mpsc::Sender<PlanetToExplorer>, mpsc::Receiver<PlanetToExplorer>) = mpsc::channel();
        let planet = Planet::new(
            2,
            PlanetType::C,
            toy_struct,
            vec![BasicResourceType::Silicon],
            vec![ComplexResourceType::Diamond, ComplexResourceType::AIPartner],
            ( orchestrator_to_planet_receiver, planet_to_orchestrato_sender ),
            (explorer_to_planet_receiver, planet_to_explorer_sender)
        );
        assert!(planet.is_ok(), "Error on creating the planet");
        let mut unwrapped_planet = planet.unwrap();
        let thread_plane = thread::spawn(move|| {
            unwrapped_planet.run();
        });
    }
}