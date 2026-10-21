// This is a crazy incantation that just... works? Thanks Claude
export type StringEnum<T extends string> = `${T}` extends T ? never : T;
