mod path_router;

use std::sync::{Arc, Mutex};

use path_router::Tree;
use rustler::{Encoder, Env, ResourceArc, Term};

mod atoms {
    rustler::atoms! {
        ok,
        error,

        route_not_found,

        unknown // Other error
    }
}

struct StringResource {
    router: Mutex<Tree<'static, u32>>,
    strings: Mutex<Vec<String>>,
}

impl StringResource {
    fn new() -> Self {
        StringResource {
            router: Mutex::new(Tree::new()),
            strings: Mutex::new(Vec::new()),
        }
    }

    fn add_string(&self, string: String) -> Arc<String> {
        let mut strings = self.strings.lock().unwrap();
        strings.push(string);
        Arc::new(strings.last().unwrap().clone())
    }

    fn add_route(&self, path: String, value: u32) {
        let arc_path = self.add_string(path);
        let path_ref: &'static str = Box::leak(Box::new((*arc_path).clone()));

        self.router.lock().unwrap().add(path_ref, value);
    }

    fn match_route<'a>(&'a self, path: &'a str) -> Option<(u32, Vec<(&'a str, &'a str)>)> {
        let router = self.router.lock().unwrap();
        router
            .find(path)
            .map(|(value, captures)| (*value, captures))
    }

    fn get_strings(&self) -> Vec<String> {
        self.strings.lock().unwrap().clone()
    }
}

fn load<'a>(env: Env<'a>, _info: Term) -> bool {
    rustler::resource!(StringResource, env);
    true
}

#[rustler::nif]
fn new() -> ResourceArc<StringResource> {
    ResourceArc::new(StringResource::new())
}

#[rustler::nif]
fn add_route(resource: ResourceArc<StringResource>, path: String, value: u32) {
    resource.add_route(path, value);
}

#[rustler::nif]
fn match_route<'a>(env: Env<'a>, resource: ResourceArc<StringResource>, path: String) -> Term<'a> {
    match resource.match_route(&path) {
        Some((value, captures)) => {
            let captures_map: Vec<(String, String)> = captures
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            (atoms::ok(), value, captures_map).encode(env)
        }
        None => atoms::route_not_found().encode(env),
    }
}

#[rustler::nif]
fn get_strings(resource: ResourceArc<StringResource>) -> Vec<String> {
    resource.get_strings()
}

rustler::init!(
    "Elixir.PathRouter.Native",
    [new, add_route, match_route, get_strings],
    load = load
);
