use crate::fish_ports::*;
use crate::makepad_micro_serde::*;
use crate::fish_param_storage::*;
use crate::fish_block::*;

#[derive(Clone,Debug, SerRon, DeRon, Default)]

pub enum FishBlockCategory
{
    Meta,
    Generator,
    Modulator, 
    Effect,
    Filter,
    Envelope,
    #[default]   Utility
}
#[derive(Clone,Debug, SerRon, DeRon, Default)]

pub struct FishBlockTemplate{
    
    pub id: u64,
    pub name: String,
    pub displayname: String,
    pub description: String,
    pub creator: String,
    pub path: String,
    pub category: FishBlockCategory,
   
   pub parameters: Vec<FishParamStorage>,
   pub inputs: Vec<FishInputPort>,
   pub outputs: Vec<FishOutputPort>,
}

#[derive(Clone, Debug, SerRon, DeRon, Default)]

pub struct FishBlockLibrary{
    pub allblocks: Vec<FishBlockTemplate>,
    pub nulltemplate: FishBlockTemplate
}

impl FishBlockLibrary
{

    pub fn populate_library(&mut self, basepath: &str)
    {
        self.nulltemplate =  FishBlockTemplate{
           category: FishBlockCategory::Meta,
            outputs: vec![],
            inputs: vec![],
            parameters: vec![],
            
            id: 0, name: String::from("Unknown"), displayname: String::from("Unknown"), description:String::from("This is the empty null block. Is something missing in your library?"), creator: String::from("Stijn Haring-Kuipers"),  path:String::from("/null") };
    
        self.allblocks.push(FishBlockTemplate{ category: FishBlockCategory::Generator,
             outputs: vec![],
             inputs: vec![],
             parameters: vec![],
             
             id: 0, name: String::from("Oscillator"), displayname: String::from("Oscillator"), description:String::from("Generic osc!"), creator: String::from("Stijn Haring-Kuipers"),  path:String::from("/oscillator") }
        ) ;
        self.allblocks.push(FishBlockTemplate{ category: FishBlockCategory::Effect,
            outputs: vec![],
            inputs: vec![],
            parameters: vec![],
            
            id: 0, name: String::from("Effect"), displayname: String::from("Effect"), description:String::from("Generic effect!"), creator: String::from("Stijn Haring-Kuipers"),  path:String::from("/effect") }
       ) ;

       self.allblocks.push(FishBlockTemplate{ category: FishBlockCategory::Filter,
        outputs: vec![],
        inputs: vec![],
        parameters: vec![],
        
        id: 0, name: String::from("Filter"), displayname: String::from("Filter"), description:String::from("Generic filter!"), creator: String::from("Stijn Haring-Kuipers"),  path:String::from("/filter") }
   ) ;
   self.allblocks.push(FishBlockTemplate{ category: FishBlockCategory::Meta,
    outputs: vec![],
    inputs: vec![],
    parameters: vec![],
    
    id: 0, name: String::from("Meta"), displayname: String::from("Meta"), description:String::from("Generic meta!"), creator: String::from("Stijn Haring-Kuipers"),  path:String::from("/meta") }
) ;

self.allblocks.push(FishBlockTemplate{ category: FishBlockCategory::Utility,
    outputs: vec![],
    inputs: vec![],
    parameters: vec![],
    
    id: 0, name: String::from("Utility"), displayname: String::from("Utility"), description:String::from("Generic utility!"), creator: String::from("Stijn Haring-Kuipers"),  path:String::from("/util") }
) ;
self.allblocks.push(FishBlockTemplate{ category: FishBlockCategory::Envelope,
    outputs: vec![],
    inputs: vec![],
    parameters: vec![],
    
    id: 0, name: String::from("Envelope"), displayname: String::from("Envelope"), description:String::from("Generic envelope!"), creator: String::from("Stijn Haring-Kuipers"),  path:String::from("/envelope") }
) ;

    }
    pub fn find_template(&self, name: &str) -> &FishBlockTemplate
    {
        if let Some(result) = self.allblocks.iter().find(|v| v.name == name){
            return result;
        }
        return &self.nulltemplate;
    }

    pub fn create_instance_from_template(&self, name: &str) -> FishBlock
    {

        let t = self.find_template(name);
        let mut f = FishBlock::default();
        f.category = t.category.clone();
        f.library_id = t.id.clone();
        f
    }
}