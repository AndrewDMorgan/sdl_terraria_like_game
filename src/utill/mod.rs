
#[derive(bincode::Encode, bincode::Decode)]
pub enum Union<A, B> {
    A(A),
    B(B),
}

