use common_game::components::energy_cell::EnergyCell;
use common_game::components::planet::*;
use common_game::components::resource::{Combinator, Generator};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::messages::*;
use common_game::protocols::messages::{
    ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
};
use std::sync::Arc;
use std::time::SystemTime;

struct CargonautsPlanet;

impl PlanetAI for CargonautsPlanet {
    fn handle_orchestrator_msg(
        &mut self,
        state: &mut PlanetState,
        msg: OrchestratorToPlanet,
    ) -> Option<PlanetToOrchestrator> {
        match msg {
            // Charge single cell at vector (position 0 because the planet is of Type C
            OrchestratorToPlanet::Sunray(sunray) => {
                let cell = state.cell_mut(0);
                cell.charge(sunray);

                Some(PlanetToOrchestrator::SunrayAck {
                    planet_id: state.id(),
                    timestamp: SystemTime::now(),
                })
            }

            // Use the method to be implemented later
            OrchestratorToPlanet::Asteroid(_) => {
                let maybe_rocket = self.handle_asteroid(state);

                Some(PlanetToOrchestrator::AsteroidAck {
                    planet_id: state.id(),
                    rocket: maybe_rocket,
                })
            }

            //same here and for stop planetAi
            OrchestratorToPlanet::StartPlanetAI(_) => {
                self.start(state);

                Some(PlanetToOrchestrator::StartPlanetAIResult {
                    planet_id: state.id(),
                    timestamp: SystemTime::now(),
                })
            }

            OrchestratorToPlanet::StopPlanetAI(_) => {
                self.stop();

                Some(PlanetToOrchestrator::StopPlanetAIResult {
                    planet_id: state.id(),
                    timestamp: SystemTime::now(),
                })
            }

            OrchestratorToPlanet::InternalStateRequest(_) => {
                todo!(
                    "Waiting for upstream fix: PlanetState allows no cloning nor manual construction"
                );
            }
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
