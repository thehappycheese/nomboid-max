use std::array;

use bevy::prelude::*;

#[derive(Clone)]
struct ProximityGridCell<const SLOT_COUNT:usize=10>{
    slots:[Option<Entity>; SLOT_COUNT]
}
impl <const SLOT_COUNT:usize> Default for ProximityGridCell<SLOT_COUNT> {
    fn default() -> Self {
        ProximityGridCell { slots: array::from_fn(|_|None) }
    }
}

impl <const SLOT_COUNT:usize> ProximityGridCell<SLOT_COUNT>{
    fn try_insert(&mut self, item:Entity)->Option<usize>{
        for index in 0..self.slots.len(){
            if self.slots[index].is_none() {
                self.slots[index]=Some(item);
                return Some(index)
            }
        }
        None
    }
    fn try_remove(&mut self, item:Entity) -> bool {
        for index in 0..self.slots.len(){
            match self.slots[index] {
                Some(x) if x==item =>{
                    self.slots[index]=None;
                    return true
                },
                _=>{}
            }
        }
        false
    }
    pub fn count(&self) -> usize {
        self.slots.iter().fold(0, |a,b| a+match b {Some(_)=>1,_=>0})
    }
}


#[derive(Resource)]
pub struct ProximityGrid<const WIDTH:usize, const HEIGHT:usize, Marker=()> where [(); WIDTH*HEIGHT]:Sized {
    grid:[ProximityGridCell; WIDTH*HEIGHT],
    _marker:std::marker::PhantomData<Marker>
}

impl <const WIDTH:usize, const HEIGHT:usize, Marker> ProximityGrid<WIDTH, HEIGHT, Marker> where [(); WIDTH*HEIGHT]:Sized {
    pub fn new()->Self{
        Self{
            grid:std::array::from_fn(|_| ProximityGridCell::default()),
            _marker:std::marker::PhantomData
        }
    }
    fn get_index_from_cell_coordinates(&self, cell_x:usize, cell_y:usize) -> Option<usize> {
        let result = cell_x+cell_y*WIDTH;
        if result< WIDTH*HEIGHT {
            Some(result)
        }else{
            None
        }
    }
    pub fn try_insert(&mut self, item:Entity, cell_x:usize, cell_y:usize)->Option<(usize,usize)> {
        self
            .get_index_from_cell_coordinates(cell_x, cell_y)
            .and_then(|index| self.grid[index].try_insert(item))
            .and_then(|_|Some((cell_x,cell_y)))
    }
    pub fn try_remove(&mut self, item:Entity, cell_x:usize, cell_y:usize)->bool {
        match self.get_index_from_cell_coordinates(cell_x, cell_y) {
            Some(index)=>self.grid[index].try_remove(item),
            _=>false
        }
    }

    ///
    /// Includes self cell
    /// 
    pub fn get_neighboring_cell_coordinates(&self, cell_x:usize, cell_y:usize) -> impl Iterator<Item=(usize, usize)> {
        (-1i64..=1i64).flat_map(|x| (-1i64..=1i64).map(move |y| (x,y)))
        .filter(move |(x,y)|!(
            // (*x==0 && *y==0) || 
            (*x<0 && cell_x==0) || 
            (*y<0 && cell_y==0) || 
            (*x>0 && cell_x== WIDTH-1) ||
            (*y>0 && cell_y==HEIGHT-1)
        ))
        .map(move |(x,y)|((cell_x as i64 + x) as usize, (cell_y as i64 + y) as usize))
    }
    pub fn get_neighbors_from_cell_coordinates(&self, cell_x:usize, cell_y:usize) -> impl Iterator<Item=Entity>{
        self
            .get_neighboring_cell_coordinates(cell_x, cell_y)
            .flat_map(|(cell_x, cell_y)|self.get_index_from_cell_coordinates(cell_x, cell_y))
            .flat_map(|index|self.grid[index].slots).flatten()
    }
    pub fn print_counts(&self){
        for y in 0..HEIGHT{
            for x in 0..WIDTH {
                let index = self.get_index_from_cell_coordinates(x, y).unwrap();
                let count = self.grid[index].count();
                print!("{count:>2}");
            }
            println!("");
        }
    }
}

#[derive(Component)]
pub struct ProximityGridStatus {
    pub inserted_at:Option<(usize, usize)>
}