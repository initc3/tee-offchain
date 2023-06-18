export declare enum AccessType {
    UNDEFINED = "UNDEFINED",
    NOBODY = "NOBODY",
    ONLY_ADDRESS = "ONLY_ADDRESS",
    EVERYBODY = "EVERYBODY"
}
export declare type AccessTypeParam = {
    value?: AccessType;
};
export declare type CodeInfo = {
    code_hash?: Uint8Array;
    creator?: Uint8Array;
    source?: string;
    builder?: string;
};
export declare type ContractCustomInfo = {
    enclave_key?: Uint8Array;
    label?: string;
};
export declare type ContractInfo = {
    code_id?: string;
    creator?: Uint8Array;
    label?: string;
    created?: AbsoluteTxPosition;
    ibc_port_id?: string;
};
export declare type AbsoluteTxPosition = {
    block_height?: string;
    tx_index?: string;
};
export declare type Model = {
    Key?: Uint8Array;
    Value?: Uint8Array;
};
//# sourceMappingURL=types.pb.d.ts.map