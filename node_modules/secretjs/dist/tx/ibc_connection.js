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
exports.MsgConnectionOpenConfirm = exports.MsgConnectionOpenAck = exports.MsgConnectionOpenTry = exports.MsgConnectionOpenInit = void 0;
/** MsgConnectionOpenInit defines the msg sent by an account on Chain A to initialize a connection with Chain B. */
class MsgConnectionOpenInit {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgConnectionOpenInit not implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgConnectionOpenInit not implemented.");
        });
    }
}
exports.MsgConnectionOpenInit = MsgConnectionOpenInit;
/** MsgConnectionOpenTry defines a msg sent by a Relayer to try to open a connection on Chain B. */
class MsgConnectionOpenTry {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgConnectionOpenTry not implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgConnectionOpenTry not implemented.");
        });
    }
}
exports.MsgConnectionOpenTry = MsgConnectionOpenTry;
/** MsgConnectionOpenAck defines a msg sent by a Relayer to Chain A to acknowledge the change of connection state to TRYOPEN on Chain B. */
class MsgConnectionOpenAck {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgConnectionOpenAck not implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgConnectionOpenAck not implemented.");
        });
    }
}
exports.MsgConnectionOpenAck = MsgConnectionOpenAck;
/** MsgConnectionOpenConfirm defines a msg sent by a Relayer to Chain B to acknowledge the change of connection state to OPEN on Chain A. */
class MsgConnectionOpenConfirm {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgConnectionOpenConfirm not implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgConnectionOpenConfirm not implemented.");
        });
    }
}
exports.MsgConnectionOpenConfirm = MsgConnectionOpenConfirm;
//# sourceMappingURL=ibc_connection.js.map