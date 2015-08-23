#![feature(convert)]
#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate rustc_serialize;
extern crate docopt;
extern crate toml;
extern crate cogset;
extern crate postgres;

use std::cmp;
use cogset::{Dbscan, BruteScan, Euclid};
use postgres::{Connection, SslMode};
use postgres::types::FromSql;
use std::fmt::Display;
use std::env;
use docopt::Docopt;
use std::fs::File;
use std::io::prelude::*;
use std::collections::BTreeMap;

docopt!(Args derive Debug, "
ducter.

Usage: 
  ducter <config>
  ducter (-h | --help)
  ducter --version

Options:
  -h --help   Show this screen.
  --version   Show version.
  <config>    Set config path.
");

pub mod data {
    use cogset::Point;
    use std::ops::Deref;
    
    fn distance(this: &'static str, that: &'static str) -> f64 {
        0 as f64
    }
    
    pub struct Geo { lang: f64, long: f64 }

    impl Point for Geo {

        fn dist(&self, other: &Self) -> f64 {
            let left = (self.long - other.long) / (self.long + other.long);
            let right = (self.lang - other.lang) / (self.lang + other.lang);
            left * right
        }
        
        fn dist_lower_bound(&self, other: &Self) -> f64 {
            self.dist(&other) * 0.1
        }
    }

    pub struct Place { name: &'static str }

    impl Point for Place {
        
        fn dist(&self, other: &Self) -> f64 {        
            let average = (self.name.len() + other.name.len()) as f64;
            distance(self.name, other.name) / average
        }

        fn dist_lower_bound(&self, other: &Self) -> f64 {
            self.dist(other) * 0.1
        }
    }

    pub enum Location {
        Geo, Place
    }

    // impl Point for Location {

    //     fn dist(&self, other: &Self) -> f64 {
    //         match (self, other) {
    //             (Geo, _) => 0 as f64,
    //             (Place, _) => 0 as f64
    //         }
    //     }

    //     fn dist_lower_bound(&self, other: &Self) -> f64 {
    //         0 as f64
    //     }
    // }
    
    pub struct Product {
        id: u64,
        name: &'static str,
        price: f64,
        location: Location
    }

    impl Point for Product {
        
        fn dist(&self, other: &Self) -> f64 {
            // * (self.location.dist(&other.location))
            //0 as f64

            distance(self.name, other.name) * (self.price - other.price)
        }

        fn dist_lower_bound(&self, other: &Self) -> f64 {
            self.dist(other) * 0.1
        }
    }
}

fn main() {
    // load from the database
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    
    let settings = {
        let mut file = File::open(args.arg_config).unwrap();
        let mut settings = String::new();
        file.read_to_string(&mut settings);
        toml::Parser::new(settings.as_str()).parse().unwrap()
    };

    
    match env::var("ENV") {
        Ok(env) => {
            let ref locals = settings[env.as_str()] as &toml::Value::Table;
            let connection = Connection::connect(
                format!("postgres://{}:{}@{}:{}/{}",
                        locals["username"],
                        locals["password"],
                            locals["host"],
                        locals["port"],
                        locals["database"])
                    .as_str(), &SslMode::None).unwrap();
            
            let statement = connection
                .prepare("select * from product_histories")
                .unwrap();
            
            let products : Vec<data::Product> = vec![];
                    
            // for row in statement.query(&[]).unwrap() {
            //     products.push(data::Product{
            //         id: row.get(0),
            //         name: row.get(1),
                    //         price: row.get(2),
            //         location: data::Location::Place { name: "test" }
            //     });
            // }
            
            let scanner = BruteScan::new(&products);
            let mut dbscan = Dbscan::new(scanner, 0.2, 2);
            
            let clusters = dbscan.by_ref().collect::<Vec<_>>();

            println!("Clusters found: {:?}", clusters);
            
            let noise = dbscan.noise_points();
            
            println!("Noise found: {:?}", noise);
        }
        _ => panic!("No environment specified"),
    };
}
