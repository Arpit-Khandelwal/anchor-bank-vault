import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorBank } from "../target/types/anchor_bank";

describe("anchor-bank", () =>
{
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.anchorBank as Program<AnchorBank>;

  it("create account", async () =>
  {
    const [depositPda, bump] = anchor.web3.PublicKey.findProgramAddressSync([program.provider.publicKey.toBuffer(), Buffer.from("deposit")], program.programId);

    console.log("Deposit PDA: ", depositPda.toBase58());

    const tx = await program.methods.create().accounts({}).rpc();

    console.log("Transaction signature", tx);
  });

  it("deposit funds", async () =>
  {
    const [depositPda, bump] = anchor.web3.PublicKey.findProgramAddressSync([program.provider.publicKey.toBuffer(), Buffer.from("deposit")], program.programId);

    console.log("balance of PDA before deposit: ", (await program.provider.connection.getBalance(depositPda)));

    const tx = await program.methods.deposit(new anchor.BN(10_000_000)).rpc();
    console.log("Deposit Transaction signature", tx);

    console.log("balance of PDA after deposit: ", (await program.provider.connection.getBalance(depositPda)));

  });

  it("read data from deposit account", async () =>
  {
    const [depositPda, bump] = anchor.web3.PublicKey.findProgramAddressSync([program.provider.publicKey.toBuffer(), Buffer.from("deposit")], program.programId);

    const depositAccount = await program.account.piggyBank.fetch(depositPda);

    console.log("Deposit Account Data: ", depositAccount.owner.toBase58(), "\n Bump: ", depositAccount.bump.toString());
    console.log("Deposited Amount: ", await program.provider.connection.getBalance(depositPda));
  });

  it("withdraw fund", async () =>
  {
    const [accountPda, bump] = anchor.web3.PublicKey.findProgramAddressSync([program.provider.publicKey.toBuffer(), Buffer.from("deposit")], program.programId);

    const accountData = await program.account.piggyBank.fetch(accountPda);

    console.log("Withdrawing from account owned by: ", accountData.owner.toBase58());

    const accountBalanceBefore = await program.provider.connection.getBalance(accountPda);
    console.log("Account balance before withdraw: ", accountBalanceBefore);

    const tx = await program.methods.withdraw(new anchor.BN(accountBalanceBefore)).accounts({}).rpc();
    console.log("Withdraw Transaction signature", tx);

    const accountBalanceAfter = await program.provider.connection.getBalance(accountPda);
    console.log("Account balance after withdraw: ", accountBalanceAfter);
  })

});
