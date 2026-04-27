enum Filter<LeafCondition, ChildFilter> {
    Leaf(LeafCondition),
    And(Vec<ChildFilter>),
    Or(Vec<ChildFilter>),
    Not(Box<ChildFilter>),
}

type F0<LeafCondition> = Filter<LeafCondition, LeafCondition>;
type F1<LeafCondition> = Filter<LeafCondition, F0<LeafCondition>>;
type F2<LeafCondition> = Filter<LeafCondition, F1<LeafCondition>>;
type F3<LeafCondition> = Filter<LeafCondition, F2<LeafCondition>>;

enum StringOperator {
    Eq(String),
    Lt(String),
    Gt(String),
}

enum PersonFieldCondition {
    Name(StringOperator),
    Email(StringOperator),
}

type PersonFilter = F3<PersonFieldCondition>;

fn main() {
    println!("Hello, world!");
}
