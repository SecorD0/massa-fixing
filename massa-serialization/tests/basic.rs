#[cfg(test)]
mod tests {
    use massa_serialization::*;
    #[test]
    fn basic() {

        #[derive(MassaSerialization)]
        #[MassaSerializationParams(params(methods = "test", methods = "test2"))]
        pub struct Obj {
            a: i32
        }

    }
}