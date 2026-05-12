
use ftool::*;

self_ref! {
    #[derive(Debug)]
    pub struct MyData {
        pub a: String,
        pub b: Vec<i32>,
        pub c: i32,
        => 
        #[derive(Debug)]
        {
            pub text: &'a str,
            pub numbers: &'a [i32],
        }
    }
}

fn main() {

    let mut sref = declare_self_ref! {

        MyData {

            a: String::from("Rust"),

            b: vec![1, 2, 3],

            c: 42,

            => { text: &a, numbers: &b }

        }

    };
    
    update_self_ref_field!(sref, a => [text: &str], String::from("New String that reallocs"));

    

    println!("{:?}", sref);

}
