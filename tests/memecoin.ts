import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Memecoin } from "../target/types/memecoin";
const { BN } = anchor;
import { utf8} from "@coral-xyz/anchor/dist/cjs/utils/bytes";

describe("memecoin", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Memecoin as Program<Memecoin>;

  it("Is initialized!", async () => {
    // Add your test here.
    const admin = anchor.AnchorProvider.local().wallet.publicKey;
      const [config_account] = anchor.web3.PublicKey
          .findProgramAddressSync([utf8.encode('CONFIG')], program.programId);
    const tx = await program.methods
        .initializeGlobalConfigs(admin, admin, new BN(100), 100)
        .accounts({
            globalConfig: config_account,
            admin,
            systemProgram: anchor.web3.SystemProgram.programId
        })
        .rpc();
    console.log("Your transaction signature", tx);
  });
});
