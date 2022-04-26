#[cfg(test)]
mod tests {
    use massa_serialization::*;

    #[test]
    fn basic() {
        #[derive(MassaSerialization)]
        #[MassaSerializationParams(params())]
        pub struct Obj {
            a: i32,
        }

        let a = Obj { a: 3 };
        a.to_bytes_compact();
        a.from_bytes_compact();
    }

    #[test]
    fn basic_with_methods() {
        #[derive(MassaSerialization)]
        #[MassaSerializationParams(
            params(
                methods(serialize = "from_bytes_in_block", deserialize = "to_bytes_in_block")
            )
        )]
        pub struct Obj {
            a: i32,
        }

        let a = Obj { a: 3 };
        a.from_bytes_in_block();
        a.to_bytes_in_block();
    }
}
