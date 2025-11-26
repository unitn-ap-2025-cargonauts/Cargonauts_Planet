use common_game::components::planet::*;
use common_game::components::rocket::Rocket;
use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};

struct CargonautsPlanet;



impl PlanetAI for CargonautsPlanet  {
    fn handle_orchestrator_msg(&mut self, state: &mut PlanetState, msg: OrchestratorToPlanet) -> Option<PlanetToOrchestrator> {
        todo!()
    }

    fn handle_explorer_msg(&mut self, state: &mut PlanetState, msg: ExplorerToPlanet) -> Option<PlanetToExplorer> {
        todo!()
    }

    fn handle_asteroid(&mut self, state: &mut PlanetState) -> Option<Rocket> {
        todo!()
    }

    fn start(&mut self, state: &PlanetState) {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }
}