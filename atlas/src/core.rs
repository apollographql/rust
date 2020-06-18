use async_std::prelude::*;

use std::{mem::size_of, hash::Hash};
use chashmap::CHashMap;

pub trait Expr: Hash {
    type Output;
    type State: Default + Value<Self::Output>;

    fn eval(&self, core: &Core, state: &mut Self::State) -> Eval;
}

pub trait Value<T> {
    fn value(&self) -> &T;
}


pub struct Slot<T> {
    pub value: T,
    pub pending: Option<Box<dyn Future<Output=T>>>
}

pub enum Eval {
    Ready,
    Wait(Box<dyn Future<Output=()>>),
}

// go![state.item = do_something()];
// go![

// load! {
//     item = color.view;
//     text = bloom.rose();
// }


// pub trait Core {
//     fn get<E: Expr>(&mut self, expr: &E) -> E::State;
//     fn once<E: Expr>(&mut self, expr: &E) -> dyn Future<Output=E::State>;
// }

pub struct Proc<S: Default> {
    pub state: S,
}

#[derive(Copy, Clone)]
struct ProcSlot {
    data: *mut u8,
}

unsafe fn as_byte_slice<'a, T>(p: &T) -> &'a [u8] {
    std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        size_of::<T>(),
    )
}

impl ProcSlot {
    fn new<S: Default>() -> ProcSlot {
        ProcSlot {
             data: Box::into_raw(Box::new(S::default())) as *mut u8
        }
     }

    fn state<'a, S: Default>(&self) -> &'a mut S {
        unsafe { &mut *(self.data as *mut S) }
    }
}

#[derive(Default)]
pub struct Core<'a> {
    procs: CHashMap<&'a [u8], ProcSlot>
}

// read!{ x = file("/src").exists }


impl<'a> Core<'a> {
    pub fn new() -> Core<'a> { Default::default() }

    pub fn state<E: Expr>(&mut self, expr: &E) -> &'a mut E::State {
        let id = unsafe { as_byte_slice(expr) };
        let slot = match self.procs.get(id) {
            None => {
                let new = ProcSlot::new::<E::State>();
                self.procs.alter(id, |existing| {
                    Some(match existing {
                        Some(value) => value,
                        None => new,
                    })
                });
                new
            },
            Some(value) => *value
        };
        slot.state()
    }

    pub fn get<E: Expr>(&mut self, expr: E)  -> &'a E::Output where E::State: 'a {
        let mut state: &mut E::State = self.state(&expr);
        expr.eval(self, &mut state);
        state.value()
    }
}

#[derive(Default, Debug, Hash, Clone)]
struct Const<T> { value: T }

impl<T: Hash> Expr for Const<T> where T: Default + Copy {
    type Output = T;
    type State = Self;

    fn eval(&self, _core: &Core, state: &mut Self::State) -> Eval {
        state.value = self.value;
        Eval::Ready
    }
}

impl<T> Value<T> for Const<T> {
    fn value(&self) -> &T { &self.value }
}



#[test]
fn it_works() {
    let mut core = Core::new();
    let x = core.get(Const { value: 5 });
    let hello = *core.get(Const { value: "hello world" });
    assert_eq!(*x, 5);
    assert_eq!(hello, "hello world");
    println!("{}", hello);
}

#[test]
fn shares_states() {
    let mut core = Core::new();
    core.state(&Const { value: "hello" });
    core.state(&Const { value: "hello" });
    core.state(&Const { value: "world" });
    assert_eq!(core.procs.len(), 2);
    assert_eq!(
        *core.get(Const { value: "hello" }),
        "hello");
    assert_eq!(
        *core.get(Const { value: "hello" }),
        "hello");    
    assert_eq!(
        *core.get(Const { value: "world" }),
        "world");
    assert_eq!(core.procs.len(), 2);
}

#[test]
fn understands_different_node_types() {
    #[derive(Hash)]
    struct Different { value: &'static str }

    impl Default for Different {
        fn default() -> Different { Different { value: "" } }
    }

    impl Value<&'static str> for Different {
        fn value(&self) -> &&'static str { &self.value }
    }

    impl Expr for Different {
        type Output = &'static str;
        type State = Self;
    
        fn eval(&self, _core: &Core, state: &mut Self::State) -> Eval {
            state.value = self.value;
            Eval::Ready
        }
    }

    let mut core = Core::new();
    core.state(&Const { value: "hello" });
    core.state(&Different { value: "hello" });
    assert_eq!(core.procs.len(), 2);
}