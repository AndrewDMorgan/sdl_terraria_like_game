
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Union<A, B> {
    A(A),
    B(B),
}

