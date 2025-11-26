use common_game::components::planet::*;
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::messages::*;

struct CargonautsPlanet;

impl PlanetAI for CargonautsPlanet  {
    fn handle_orchestrator_msg(&mut self, state: &mut PlanetState, msg: OrchestratorToPlanet) -> Option<PlanetToOrchestrator> {
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

    fn handle_explorer_msg(&mut self, state: &mut PlanetState, msg: ExplorerToPlanet) -> Option<PlanetToExplorer> {
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id } => {
                handle_supported_resource_request(state, explorer_id)
            },
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                handle_supported_combination_request(state, explorer_id)
            },
            ExplorerToPlanet::GenerateResourceRequest { explorer_id, msg } => {
                handle_generate_resource_request(state, explorer_id, msg)
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
        }
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
    msg: CombineResourceRequest,
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
