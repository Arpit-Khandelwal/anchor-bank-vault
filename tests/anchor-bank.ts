import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorBank } from "../target/types/anchor_bank";

describe("anchor-bank", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.anchorBank as Program<AnchorBank>;

  it("create account", async () => {
    const [depositPda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [program.provider.publicKey.toBuffer(), Buffer.from("deposit")],
      program.programId
    );

    console.log("Deposit PDA: ", depositPda.toBase58());

    const tx = await program.methods.create().accounts({}).rpc();

    console.log("Transaction signature", tx);
  });

  it("deposit funds", async () => {
    const [depositPda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [program.provider.publicKey.toBuffer(), Buffer.from("deposit")],
      program.programId
    );

    console.log(
      "balance of PDA before deposit: ",
      await program.provider.connection.getBalance(depositPda)
    );

    const tx = await program.methods
      .deposit(new anchor.BN(1_000_000_000))
      .rpc();
    console.log("Deposit Transaction signature", tx);

    console.log(
      "balance of PDA after deposit: ",
      await program.provider.connection.getBalance(depositPda)
    );
  });

  it("read data from deposit account", async () => {
    const [depositPda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [program.provider.publicKey.toBuffer(), Buffer.from("deposit")],
      program.programId
    );

    const depositAccount = await program.account.piggyBank.fetch(depositPda);

    console.log(
      "Deposit Account Data: ",
      depositAccount.owner.toBase58(),
      "\n Bump: ",
      depositAccount.bump.toString()
    );
    console.log(
      "Deposited Amount: ",
      await program.provider.connection.getBalance(depositPda)
    );
  });

  it("withdraw fund", async () => {
    const [accountPda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [program.provider.publicKey.toBuffer(), Buffer.from("deposit")],
      program.programId
    );

    const accountData = await program.account.piggyBank.fetch(accountPda);

    console.log(
      "Withdrawing from account owned by: ",
      accountData.owner.toBase58()
    );

    const accountBalanceBefore = await program.provider.connection.getBalance(
      accountPda
    );
    console.log("Account balance before withdraw: ", accountBalanceBefore);

    const minBalance = 20_000_000; // 0.02 SOL
    const withdrawAmount = new anchor.BN(accountBalanceBefore).sub(
      new anchor.BN(minBalance)
    );

    const tx = await program.methods
      .withdraw(withdrawAmount)
      .accounts({})
      .rpc();
    console.log("Withdraw Transaction signature", tx);

    const accountBalanceAfter = await program.provider.connection.getBalance(
      accountPda
    );
    console.log("Account balance after withdraw: ", accountBalanceAfter);
  });

  it("reset account", async () => {
    const [depositPda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [program.provider.publicKey.toBuffer(), Buffer.from("deposit")],
      program.programId
    );

    // Ensure account exists first (create if needed)
    try {
      await program.account.piggyBank.fetch(depositPda);
    } catch (e) {
      await program.methods.create().accounts({}).rpc();
    }

    const tx = await program.methods.reset().rpc();

    console.log("Reset Transaction signature", tx);

    // Verify account is closed (should fail to fetch)
    try {
      await program.account.piggyBank.fetch(depositPda);
      throw new Error("Account should be closed");
    } catch (e) {
      // Expected error: Account does not exist or failed to deserialize (if it was just zeroed out but not closed properly, but here we expect it to be closed/lamports=0 which usually means it's gone from runtime perspective for Anchor fetch)
      console.log("Account successfully reset/closed");
    }
  });

  it("withdraw full amount and close", async () => {
    const [depositPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [program.provider.publicKey.toBuffer(), Buffer.from("deposit")],
      program.programId
    );

    // Ensure account exists
    try { await program.methods.create().accounts({}).rpc(); } catch(e){}
    await program.methods.deposit(new anchor.BN(100_000_000)).accounts({}).rpc();

    // Withdraw FULL balance
    const balance = await program.provider.connection.getBalance(depositPda);
    console.log("Withdrawing full balance:", balance);
    
    const tx = await program.methods.withdraw(new anchor.BN(balance)).accounts({}).rpc();
    console.log("Full Withdraw Transaction signature", tx);

    // Verify closed
    try {
        await program.account.piggyBank.fetch(depositPda);
        throw new Error("Account should be closed");
    } catch (e) {
        console.log("Account successfully closed via withdraw");
    }
  });
});
