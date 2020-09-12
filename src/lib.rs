pub mod draw;

pub use golem;
pub use golem::glow;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
