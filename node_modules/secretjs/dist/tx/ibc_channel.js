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
exports.MsgChannelCloseConfirm = exports.MsgChannelCloseInit = exports.MsgChannelOpenConfirm = exports.MsgChannelOpenAck = exports.MsgChannelOpenTry = exports.MsgAcknowledgement = exports.MsgChannelOpenInit = exports.MsgTimeoutOnClose = exports.MsgTimeout = exports.MsgRecvPacket = void 0;
/** MsgRecvPacket receives incoming IBC packet */
class MsgRecvPacket {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgRecvPacket not implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgRecvPacket not implemented.");
        });
    }
}
exports.MsgRecvPacket = MsgRecvPacket;
/** MsgTimeout receives timed-out packet */
class MsgTimeout {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgTimeout not implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgTimeout not implemented.");
        });
    }
}
exports.MsgTimeout = MsgTimeout;
/** MsgTimeoutOnClose timed-out packet upon counterparty channel closure. */
class MsgTimeoutOnClose {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("Method MsgTimeoutOnClose implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgTimeoutOnClose not implemented.");
        });
    }
}
exports.MsgTimeoutOnClose = MsgTimeoutOnClose;
/**
 * MsgChannelOpenInit defines an sdk.Msg to initialize a channel handshake. It
 * is called by a relayer on Chain A.
 */
class MsgChannelOpenInit {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("Method MsgChannelOpenInit implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgChannelOpenInit not implemented.");
        });
    }
}
exports.MsgChannelOpenInit = MsgChannelOpenInit;
/** MsgAcknowledgement receives incoming IBC acknowledgement */
class MsgAcknowledgement {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("Method MsgAcknowledgement implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgAcknowledgement not implemented.");
        });
    }
}
exports.MsgAcknowledgement = MsgAcknowledgement;
/**
 * MsgChannelOpenInit defines a msg sent by a Relayer to try to open a channel
 * on Chain B.
 */
class MsgChannelOpenTry {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("Method MsgChannelOpenTry implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgChannelOpenTry not implemented.");
        });
    }
}
exports.MsgChannelOpenTry = MsgChannelOpenTry;
/**
 * MsgChannelOpenAck defines a msg sent by a Relayer to Chain A to acknowledge
 * the change of channel state to TRYOPEN on Chain B.
 */
class MsgChannelOpenAck {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("Method MsgChannelOpenAck implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgChannelOpenAck not implemented.");
        });
    }
}
exports.MsgChannelOpenAck = MsgChannelOpenAck;
/**
 * MsgChannelOpenConfirm defines a msg sent by a Relayer to Chain B to
 * acknowledge the change of channel state to OPEN on Chain A.
 */
class MsgChannelOpenConfirm {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("Method not MsgChannelOpenConfirm.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgChannelOpenConfirm not implemented.");
        });
    }
}
exports.MsgChannelOpenConfirm = MsgChannelOpenConfirm;
/**
 * MsgChannelCloseInit defines a msg sent by a Relayer to Chain A
 * to close a channel with Chain B.
 */
class MsgChannelCloseInit {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("Method MsgChannelCloseInit implemented.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgChannelCloseInit not implemented.");
        });
    }
}
exports.MsgChannelCloseInit = MsgChannelCloseInit;
/** MsgChannelCloseConfirm defines a msg sent by a Relayer to Chain B to acknowledge the change of channel state to CLOSED on Chain A. */
class MsgChannelCloseConfirm {
    constructor(msg) { }
    toProto() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("Method not MsgChannelCloseConfirm.");
        });
    }
    toAmino() {
        return __awaiter(this, void 0, void 0, function* () {
            throw new Error("MsgChannelCloseConfirm not implemented.");
        });
    }
}
exports.MsgChannelCloseConfirm = MsgChannelCloseConfirm;
//# sourceMappingURL=ibc_channel.js.map