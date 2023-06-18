import { SecretNetworkClient, Wallet } from "secretjs";
import * as fs from "fs";
import dotenv from "dotenv";
dotenv.config();

const wallet = new Wallet("miracle cash equal flee lawsuit buffalo victory city relax arrange voice night toilet guilt congress badge reject random fly puzzle bone mystery ugly similar");
// secret1aw22y0wfwhythd06w2c5375q8hrg2jw642d8ks

// Node messaging url
const nodeUrl = `http://localhost:${process.env.SECRET_PORT_NUMBER}`;
export const secretjs = new SecretNetworkClient({
    chainId: "secretdev-1",
    url: nodeUrl,
    wallet: wallet,
    walletAddress: wallet.address,
});