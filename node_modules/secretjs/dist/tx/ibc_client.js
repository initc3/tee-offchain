"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.MsgCreateClient = exports.MsgSubmitMisbehaviour = exports.MsgUpgradeClient = exports.MsgUpdateClient = void 0;
/** MsgUpdateClient defines an sdk.Msg to update a IBC client state using the given header. */
class MsgUpdateClient {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgUpdateClient not implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgUpdateClient not implemented.");
        });
    }
}
exports.MsgUpdateClient = MsgUpdateClient;
/** MsgUpdateClient defines an sdk.Msg to update a IBC client state using the given header. */
class MsgUpgradeClient {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgUpgradeClient not implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgUpgradeClient not implemented.");
        });
    }
}
exports.MsgUpgradeClient = MsgUpgradeClient;
/** MsgSubmitMisbehaviour defines an sdk.Msg type that submits Evidence for light client misbehaviour. */
class MsgSubmitMisbehaviour {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgSubmitMisbehaviour not implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgSubmitMisbehaviour not implemented.");
        });
    }
}
exports.MsgSubmitMisbehaviour = MsgSubmitMisbehaviour;
/** MsgCreateClient defines a message to create an IBC client */
class MsgCreateClient {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgCreateClient not implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgCreateClient not implemented.");
        });
    }
}
exports.MsgCreateClient = MsgCreateClient;
//# sourceMappingURL=ibc_client.js.map