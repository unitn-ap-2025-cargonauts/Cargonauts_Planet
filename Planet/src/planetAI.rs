//! # Cargonauts Planet AI Module
//!
//! This module contains the implementation of the `PlanetAI` trait for the
//! Cargonauts planet. It defines how the planet handle messages from the
//! Orchestrator and the Explorer.
//!
//! Each handler is defined as a standalone function to keep the logic modular and clean.

use std::fmt::{Debug, Display, Formatter};
use common_game::components::planet::*;
use common_game::components::resource::*;
use common_game::components::rocket::Rocket;
use common_game::protocols::messages::*;


trait PlanetDefinition {
    fn get_name(&self) -> &'static str;
    fn get_type(&self) -> &'static PlanetType;
}

#[derive(Debug)]
enum PlanetAIBehavior {
    Survival,
    Normal
}

struct CargonautsPlanet {
    ai_is_active: bool,
    ai_mode: PlanetAIBehavior,
    cached_basic_resource: Vec<BasicResourceType>
}

impl PlanetDefinition for CargonautsPlanet {
    fn get_name(&self) -> &'static str {
        "Cargonauts Planet"
    }

    fn get_type(&self) -> &'static PlanetType {
        &PlanetType::C
    }
}


impl Default for CargonautsPlanet {
    fn default() -> Self {
        Self {
            ai_is_active: true,
            ai_mode: PlanetAIBehavior::Survival,
            cached_basic_resource : vec![]
        }
    }
}


impl Debug for CargonautsPlanet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Planet name: {} {{ {:?}, {:?} }}", self.get_name() , self.ai_is_active, self.ai_mode )
    }
}


impl Display for CargonautsPlanet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Planet {}, type: {:?}", self.get_name(), self.get_type())
    }
}


impl CargonautsPlanet {

    //! Set the PlanetAI behavior as the one defined in the method argument
    //!
    //! # Parameters
    //! - `new_mode : PlanetAIBehavior` the new state behavior that must be set
    fn switch_mode(&mut self, new_mode: PlanetAIBehavior) {
        self.ai_mode = new_mode;
    }


}

struct ResourcesCache {
    basic_cache: Vec<BasicResourceType>,
    complex_cache: Vec<ComplexResourceType>
}


impl ResourcesCache {


    fn new() -> Self {
        Self {
            basic_cache: Vec::new(),
            complex_cache: Vec::new()
        }
    }

    fn add_in_cache(&mut self, new_resource: ResourceType) {
        match new_resource {
            ResourceType::Basic(basic_res) => {
                self.basic_cache.push( basic_res )
            },
            ResourceType::Complex(complex_res) => {
                self.complex_cache.push( complex_res )
            }
        }
    }

    fn get_resource(&mut self, resource: ResourceType) -> Option<ResourceType> {

        match resource {
            ResourceType::Basic(basic_res) => {

                let basic_in = self.basic_cache.contains( &basic_res );
                if basic_in {
                    let position = self.basic_cache.iter().position(| x | x.eq(&basic_res));
                    Some(ResourceType::Basic( self.basic_cache.remove(position.unwrap()) ))
                } else {
                    None
                }
            },
            ResourceType::Complex(complex_res ) => {
                let basic_in = self.complex_cache.contains( &complex_res );
                if basic_in {
                    let position = self.complex_cache.iter().position(| x | x.eq(&complex_res));
                    Some(ResourceType::Complex( self.complex_cache.remove(position.unwrap()) ))
                } else {
                    None
                }
            }
        }
    }

    fn get_basic_cache(&self) -> &Vec<BasicResourceType> {
        self.basic_cache.as_ref()
    }

    fn get_complex_cache(&self) -> &Vec<ComplexResourceType> {
        self.complex_cache.as_ref()
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

                // Check if the energy cell should be charged
                if !state.cell(0).is_charged() {
                    state.charge_cell(sunray);
                }

                match self.ai_mode {
                    PlanetAIBehavior::Survival => {
                        // Surely at this point the energy cell is charged
                        // TODO: an assumption here is that the planet does not have a Rocket
                        //  otherwise it would not be there
                        let _ = state.build_rocket( 0 );

                        // Switch to Normal mode
                        self.switch_mode( PlanetAIBehavior::Normal );

                        Some(PlanetToOrchestrator::SunrayAck { planet_id: state.id() })
                    },
                    PlanetAIBehavior::Normal => {
                        // todo!("Honest idk what to do with that. I think i should produce stuff but\
                        // I am not sure i Can ")
                        let generated_carbon = generator.make_carbon( state.cell_mut(0) );
                        Some(PlanetToOrchestrator::SunrayAck {planet_id : state.id()})
                    }
                }
            }

            // Use the method to be implemented later
            OrchestratorToPlanet::Asteroid(_) => {
                Some(PlanetToOrchestrator::AsteroidAck {
                    planet_id: state.id(),
                    rocket: self.handle_asteroid(state, generator, combinator),
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
                //info!("SupportedResourceRequest received from explorer[{}]", explorer_id);
                handle_supported_resource_request(generator)
            },
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                //info!("SupportedCombinationRequest received from explorer[{}]", explorer_id);
                handle_supported_combination_request(combinator)
            },
            ExplorerToPlanet::GenerateResourceRequest { explorer_id, resource } => {
                //info!("GenerateResourceRequest received from explorer[{}]. Ask for generate {:?}", explorer_id, resource);
                handle_generate_resource_request(state, generator, resource)
            },
            ExplorerToPlanet::CombineResourceRequest { explorer_id, msg } => {
                //info!("CombineResourceRequest received from explorer[{}]. Ask for craft {:?}", explorer_id, msg);
                handle_combine_resource_request(state, combinator, msg)
            },
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id } => {
                //info!("AvailableEnergyCellRequest received from explorer[{}]", explorer_id);
                handle_energy_cell_request(state)
            },
            _ => panic!("Unexpected message")
        }
    }


    /// Handler for the [Asteroid] message, it returns `None` or `Some(Rocket)` based on the rules of the
    /// [Planet] or the availability of [Rocket] on the planet. 
    ///
    /// More precisely, it returns `None` if:
    /// - The [Planet] can not create any [Rocket] because of its rules.
    /// - The [Planet] can not crate any [Rocket] because it has no charged [EnergyCell].
    ///
    /// It returns `Some(Rocket)` if:
    /// - There already is a [Rocket] that can be used.
    /// - The planet is able to build a [Rocket]
    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _: &Generator,
        _: &Combinator
    ) -> Option<Rocket> {

        // Drop the hanlder if the AI is not active
        // TODO: technically if AI is disable I should not be able to reach this since the
        //  function (right now) is built in a way that i can not arrive there in case of disabled AI
        if !self.ai_is_active {
            return None;
        }

        // At this point the Rocket can be built. Check if there already
        // is a rocket ready to be used
        if state.has_rocket() {
            let rocket = state.take_rocket().unwrap();

            // Switch to survival mode
            self.switch_mode( PlanetAIBehavior::Survival );

            // Send a warn to the explorer
            // TODO : this is not possible since I cannot directly talk to the explorer from there

            // Finally, return the rocket
            Some(rocket)
        } else {
            // The rocket is not available, check if it still can be created with the use of an
            // EnergyCell.
            let charged_cell_position = state.cells_iter().position( |cell_ref| cell_ref.is_charged() );
            if let Some(charged_cell_position_result) = charged_cell_position {
                // Create the rocket and return it
                let created_rocket_result = state.build_rocket( charged_cell_position_result );
                return if let Ok(_) = created_rocket_result {
                    // Switch mode and return the rocket
                    self.switch_mode(PlanetAIBehavior::Survival);
                    state.take_rocket()
                } else {
                    // q1 on obsidian: the error can happen either because of it does not have
                    // any free cell or it already has a rocket which should not be happen since
                    // I have already checked this before
                    None
                }
            }
            // Rocket can not be built but this should not be possible
            // TODO: log the logical error
            None
        }
    }

    fn start(&mut self, state: &PlanetState) {
        self.ai_is_active = true;
    }

    fn stop(&mut self, state: &PlanetState) {
        self.ai_is_active = true;
    }
}

// === OrchestratorToPlanet Handler ================================================================


// === ExplorerToPlanet Handler ====================================================================
/// This handler returns a `SupportedResourceResponse` message that wrap the list of basic resources
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
/// - Wrap the set in a `SupportedResourceResponse` message and return it
fn handle_supported_resource_request(
    generator: &Generator,
) -> Option<PlanetToExplorer> {
    let resource_list = generator.all_available_recipes();
    Some(PlanetToExplorer::SupportedResourceResponse { resource_list })
}

/// This handler returns a `SupportedCombinationResponse` message that wrap the list of complex resources
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
/// - Wrap the set in a `SupportedCombinationResponse` message and return it
fn handle_supported_combination_request(
    combinator: &Combinator,
) -> Option<PlanetToExplorer> {
    let combination_list = combinator.all_available_recipes();
    Some(PlanetToExplorer::SupportedCombinationResponse  { combination_list })
}

/// This handler processes a request to generate a basic resource using the planet's generator, 
/// if energy is available.
/// It returns a `GenerateResourceResponse` message containing the generated resource.
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
///     - Wrap the generated resource in a `GenerateResourceResponse` message and return it.
/// - Else:
///     - Wrap a `None` in a `GenerateResourceResponse` message and return it.
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

/// This handler processes a request to combine two basic or complex resources
/// into a new complex resource using the planet's combinator, if energy is available.
/// It returns a `CombineResourceResponse` message containing the newly
/// crafted complex resource.
///
/// # Parameters
/// - `state`: Mutable reference to the planet state.
/// - `combinator`: Reference to the planet's combinator.
/// - `msg`: A `ComplexResourceRequest` specifying which complex resource the
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
///     - Wrap the produced resource in a `CombineResourceResponse` message and return it.
/// - Else:
///     - Wrap a `None` in a `CombineResourceResponse` message and return it.
fn handle_combine_resource_request(
    state: &mut PlanetState,
    combinator: &Combinator,
    msg: ComplexResourceRequest,
) -> Option<PlanetToExplorer> {
    let mut complex_response: Option<ComplexResource> = None;
    let energy_cell = state.cell_mut(0);
    if energy_cell.is_charged(){
        match msg {
            ComplexResourceRequest::Diamond(carbon1, carbon2) => {
                match combinator.make_diamond(carbon1, carbon2, energy_cell){
                    Ok( r) => complex_response = Some(ComplexResource::Diamond(r)),
                    Err(e) => panic!("{:?}", e) //TODO right?
                }
            },
            ComplexResourceRequest::Life(water, carbon) => {
                match combinator.make_life(water, carbon, energy_cell){
                    Ok( r) => complex_response = Some(ComplexResource::Life(r)),
                    Err(e) => panic!("{:?}", e) //TODO right?
                }
            },
            _ => panic!("Unexpected resource type") //TODO right?
        }
    }
    Some(PlanetToExplorer::CombineResourceResponse { complex_response })
}

/// This handler returns an `AvailableEnergyCellResponse` message containing
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
/// - Wraps the counter inside an `AvailableEnergyCellResponse` message and returns it
fn handle_energy_cell_request(
    state: &PlanetState,
) -> Option<PlanetToExplorer> {
    let mut available_cells = 0;
    if state.cell(0).is_charged() {
        available_cells += 1;
    }
    Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells })
}

///TODO fn handle_internal_state_request description
fn handle_internal_state_request(
    state: &PlanetState,
    explorer_id: u32,
) -> Option<PlanetToExplorer> {
    /*Some(PlanetToExplorer::InternalStateResponse { planet_state: state })*/ //TODO find out the utility of this msg and ask if is need to pass the only ref and not the whole ownership
    todo!()
}




#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    //use std::collections::HashSet;
    use std::thread;
    use common_game::components::asteroid::Asteroid;
    use common_game::components::sunray::Sunray;
    use common_game::components::resource::{BasicResourceType, ComplexResourceType};
    use common_game::components::planet::{Planet, PlanetAI, PlanetType};
    use common_game::protocols::messages::{ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator};
    use crate::planetAI::CargonautsPlanet;

    // Function that create a Planet with specific arguments
    /*fn create_planet(
        id: u32,
        planet_type: PlanetType,
        gen_rules: Vec<BasicResourceType>,
        comb_rules: Vec<ComplexResourceType>
    ) -> Planet<CargonautsPlanet> {
        let (to_orchestrator_tx, _to_orchestrator_rx) = mpsc::channel(); // Planet -> Orchestrator
        let (_from_orchestrator_tx, from_orchestrator_rx) = mpsc::channel(); // Orchestrator -> Planet
        let (to_explorer_tx, _to_explorer_rx) = mpsc::channel(); // Planet -> Explorer
        let (_from_explorer_tx, from_explorer_rx) = mpsc::channel(); // Explorer -> Planet

        Planet::new(
            id,
            planet_type,
            CargonautsPlanet::default(),
            gen_rules,
            comb_rules,
            (from_orchestrator_rx, to_orchestrator_tx),
            (from_explorer_rx, to_explorer_tx),
        ).expect("Failed to create planet")
    }*/

    fn planet_to_explorer_channel_creator() -> (mpsc::Sender<PlanetToExplorer>, mpsc::Receiver<PlanetToExplorer>) {
        let (planet_to_explorer_sender, planet_to_explorer_receiver): (mpsc::Sender<PlanetToExplorer>, mpsc::Receiver<PlanetToExplorer>) = mpsc::channel();
        (planet_to_explorer_sender, planet_to_explorer_receiver)
    }

    fn explorer_to_planet_channels_creator() -> (mpsc::Sender<ExplorerToPlanet>, mpsc::Receiver<ExplorerToPlanet>) {
        let (explorer_to_planet_sender, explorer_to_planet_receiver): (mpsc::Sender<ExplorerToPlanet>, mpsc::Receiver<ExplorerToPlanet>) = mpsc::channel();
        (explorer_to_planet_sender, explorer_to_planet_receiver)
    }

    fn orchestrator_to_planet_channels_creator() -> (mpsc::Sender<OrchestratorToPlanet>, mpsc::Receiver<OrchestratorToPlanet>) {
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver): (mpsc::Sender<OrchestratorToPlanet>, mpsc::Receiver<OrchestratorToPlanet>) = mpsc::channel();
        (orchestrator_to_planet_sender, orchestrator_to_planet_receiver)
    }

    fn planet_to_orchestrator_channels_crator() -> (mpsc::Sender<PlanetToOrchestrator>, mpsc::Receiver<PlanetToOrchestrator>) {
        let (planet_to_orchestrato_sender, planet_to_orchestrator_receiver): (mpsc::Sender<PlanetToOrchestrator>, mpsc::Receiver<PlanetToOrchestrator>) = mpsc::channel();
        (planet_to_orchestrato_sender, planet_to_orchestrator_receiver)
    }

    fn create_planet(
        (planet_to_orchestrator_sender, orchestrator_to_planet_receiver): (mpsc::Sender<PlanetToOrchestrator>, mpsc::Receiver<OrchestratorToPlanet>),
        (planet_to_explorer_sender, explorer_to_planet_receiver): (mpsc::Sender<PlanetToExplorer>, mpsc::Receiver<ExplorerToPlanet>),
        ai: Box<dyn PlanetAI>
    ) -> Planet {
        let planet = Planet::new(
            2,
            PlanetType::C,
            ai,
            vec![BasicResourceType::Silicon],
            vec![ComplexResourceType::Diamond, ComplexResourceType::AIPartner],
            (orchestrator_to_planet_receiver, planet_to_orchestrator_sender),
            (explorer_to_planet_receiver, planet_to_explorer_sender)
        );
        assert!(planet.is_ok(), "Planet creatrion error!");
        planet.unwrap()
    }

    #[test]
    fn asteroid_with_uncharged_cell() {
        let toy_struct = CargonautsPlanet::default();
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) = orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrato_sender, planet_to_orchestrator_receiver) = planet_to_orchestrator_channels_crator();

        let (_, explorer_to_planet_receiver) = explorer_to_planet_channels_creator();
        let (planet_to_explorer_sender, _) = planet_to_explorer_channel_creator();


        let mut planet = create_planet(
            (planet_to_orchestrato_sender, orchestrator_to_planet_receiver),
            (planet_to_explorer_sender, explorer_to_planet_receiver),
            Box::from(toy_struct)
        );

        // Spawn the thread:
        let _ = thread::spawn(move || {
            let _ = planet.run();
        });

        // Make the planet start
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::StartPlanetAI);

        // Send an asteroid
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::Asteroid(Asteroid::default()));
        let planet_response = planet_to_orchestrator_receiver.recv().unwrap();
        assert!(matches!( planet_response, PlanetToOrchestrator::AsteroidAck { .. } ));
        assert!(matches!( planet_response, PlanetToOrchestrator::AsteroidAck { planet_id: 2, rocket: None }));
    }


    #[test]
    fn test_asteroid_handler_with_charged_cell() {
        let toy_struct = CargonautsPlanet::default();
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) = orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrato_sender, planet_to_orchestrator_receiver) = planet_to_orchestrator_channels_crator();

        let (_, explorer_to_planet_receiver) = explorer_to_planet_channels_creator();
        let (planet_to_explorer_sender, _) = planet_to_explorer_channel_creator();


        let mut planet = create_planet(
            (planet_to_orchestrato_sender, orchestrator_to_planet_receiver),
            (planet_to_explorer_sender, explorer_to_planet_receiver),
            Box::from(toy_struct)
        );

        // Spawn the thread:
        let therad_var = thread::spawn(move || {
            let _ = planet.run();
        });

        // Make the planet start
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::StartPlanetAI);


        // Send sunrays
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::Sunray(Sunray::default()));
        let sunrays_planet_response = planet_to_orchestrator_receiver.recv();
        assert!(matches!( sunrays_planet_response.unwrap(), PlanetToOrchestrator::SunrayAck { .. } ), "Did not received a sunrays AKC");

        // Send the asteroid
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::Asteroid(Asteroid::default()));
        let planet_response = planet_to_orchestrator_receiver.recv();
        assert!(planet_response.is_ok(), "Error with the response of the planet once the Asteroid");

        if let Ok(planet_response_msg) = planet_response {
            assert!(matches!( planet_response_msg, PlanetToOrchestrator::AsteroidAck { planet_id: 2,  rocket: _ }), "Planet answered with a different ID");
            assert!(matches!( planet_response_msg, PlanetToOrchestrator::AsteroidAck { planet_id: 2,  rocket: Some( _ ) }));
            assert!(matches!(planet_response_msg, PlanetToOrchestrator::AsteroidAck { .. } ), "The planet did not answer back with a AsteroidAck");
        }
    }

    #[test]
    fn test_rocket_with_disabled_ai() {


        let toy_struct = CargonautsPlanet::default();
        let (orchestrator_to_planet_sender, orchestrator_to_planet_receiver) = orchestrator_to_planet_channels_creator();
        let (planet_to_orchestrato_sender, planet_to_orchestrator_receiver) = planet_to_orchestrator_channels_crator();

        let (_, explorer_to_planet_receiver) = explorer_to_planet_channels_creator();
        let (planet_to_explorer_sender, _) = planet_to_explorer_channel_creator();


        let mut planet = create_planet(
            (planet_to_orchestrato_sender, orchestrator_to_planet_receiver),
            (planet_to_explorer_sender, explorer_to_planet_receiver),
            Box::from(toy_struct)
        );


        // Shutdown the planet AI
        let _ = orchestrator_to_planet_sender.send( OrchestratorToPlanet::StopPlanetAI );
        let _ = planet_to_orchestrator_receiver.recv().unwrap();

        // Send the asteroid
        let _ = orchestrator_to_planet_sender.send(OrchestratorToPlanet::Asteroid(Asteroid::default()));
        let planet_response = planet_to_orchestrator_receiver.recv();
        assert!(planet_response.is_ok(), "Error with the response of the planet once the Asteroid");
        assert!( matches!(planet_response.unwrap(), PlanetToOrchestrator::AsteroidAck {planet_id: _, rocket: None}), "The AI should be stopped thus the planet should not be able to send with a rocket" );
    }

    /*#[test]
    fn test_base_handle_supported_resource_request() {
        let planet_id = 0;
        let planet_type = PlanetType::C;
        let gen_rules = vec![BasicResourceType::Carbon];
        let comb_rules = vec![];
        let planet = create_planet(planet_id, planet_type, gen_rules.clone(), comb_rules);

        let result = handle_supported_resource_request(planet.generator()); //TODO wait for the add of getter and than test

        assert!(result.is_some());

        if let Some(PlanetToExplorer::SupportedResourceResponse { resource_list }) = result {
            assert!(resource_list.is_some());
            let resource_vec = resource_list.unwrap();

            let result_set: HashSet<BasicResourceType> = resource_vec.into_iter().collect();
            let expected_set: HashSet<BasicResourceType> = gen_rules.into_iter().collect();
            assert_eq!(result_set, expected_set);
        } else {
            panic!("Expected SupportedResourceResponse variant");
        }
    }

    #[test]
    fn test_base_handle_supported_combination_request() {
        let planet_id = 0;
        let planet_type = PlanetType::C;
        let gen_rules = vec![BasicResourceType::Carbon];
        let comb_rules = vec![ComplexResourceType::Diamond, ComplexResourceType::Life];
        let planet = create_planet(planet_id, planet_type, gen_rules, comb_rules.clone());

        let result = handle_supported_combination_request(planet.combinator()); //TODO wait for the add of getter and than test

        assert!(result.is_some());

        if let Some(PlanetToExplorer::SupportedCombinationResponse { combination_list }) = result {
            assert!(combination_list.is_some());
            let resource_vec = combination_list.unwrap();

            let result_set: HashSet<ComplexResourceType> = resource_vec.into_iter().collect();
            let expected_set: HashSet<ComplexResourceType> = comb_rules.into_iter().collect();
            assert_eq!(result_set, expected_set);
        } else {
            panic!("Expected SupportedCombinationResponse variant");
        }
    }*/

    /*#[test]
    fn test_base_handle_energy_cell_request_charge() {
        todo!()
    }

    #[test]
    fn test_base_handle_energy_cell_request_discharge() {
        todo!()
    }*/
}