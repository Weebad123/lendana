import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Lendana } from "../target/types/lendana";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
  createAssociatedTokenAccount,
  getAssociatedTokenAddressSync,
  createMint,
  mintTo,
  mintToChecked,
  getAccount,
} from "@solana/spl-token";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";
import { expect } from "chai";
import { BN } from "bn.js";

describe("lendana", () => {
  // Configure the client to use the local cluster.

  /** ---------------------     TEST SETUP       -------------------- */
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Lendana as Program<Lendana>;

  const TOKEN_PROGRAM: typeof TOKEN_2022_PROGRAM_ID | typeof TOKEN_PROGRAM_ID =
    TOKEN_2022_PROGRAM_ID;

  // Actors In The System
  const deployer = provider.wallet;
  const lendanaAdmin = anchor.web3.Keypair.generate();
  const whitelister = anchor.web3.Keypair.generate();
  const lender1 = anchor.web3.Keypair.generate();

  /** AIRDROP FUNCTION */
  async function airdropSol(provider, publicKey, amountInSol) {
    const airdropSig = await provider.connection.requestAirdrop(
      publicKey,
      amountInSol * LAMPORTS_PER_SOL
    );

    await provider.connection.confirmTransaction(airdropSig);
  }

  async function setupActors(provider, users, amount) {
    for (const user of users) {
      await airdropSol(provider, user, amount);
    }
  }

  before(async () => {
    await airdropSol(provider, deployer.publicKey, 5);

    await setupActors(
      provider,
      [lendanaAdmin.publicKey, whitelister.publicKey, lender1.publicKey],
      5
    );
  });

  it("TEST 1: ---------- ADMIN INITIALIZATION DONE CORRECTLY! -----------", async () => {
    // Add your test here.
    const [adminAccountPDA, adminAccountBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("admin"), lendanaAdmin.publicKey.toBuffer()],
        program.programId
      );

    // Call The Instruction
    await program.methods
      .initializeAdmin(lendanaAdmin.publicKey)
      .accounts({})
      .signers([])
      .rpc();

    const adminData = await program.account.administrator.fetch(
      adminAccountPDA
    );
    expect(adminData.adminAddress.toBuffer()).to.deep.equal(
      lendanaAdmin.publicKey.toBuffer()
    );
  });

  it("TEST 2: ----------- INITIALIZATION OF GLOBAL TRUSTED ENTITIES DONE CORRECTLY  ----------------", async () => {
    // Get AdminAccount
    const [globalTrustedPDA, globalTrustedBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("trusted_entities")],
        program.programId
      );

    const [adminAccountPDA, adminAccountBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("admin"), lendanaAdmin.publicKey.toBuffer()],
        program.programId
      );

    // Call The Instruction
    await program.methods
      .initializeTrustedAuthority()
      .accounts({
        admin: lendanaAdmin.publicKey,
        // @ts-ignore
        adminAccount: adminAccountPDA,
        trustedRoles: globalTrustedPDA,
      })
      .signers([lendanaAdmin])
      .rpc();

    // Get Account Data
    const globalTrustedData = await program.account.trustedEntities.fetch(
      globalTrustedPDA
    );
    expect(globalTrustedData.trustedRoles.length).to.eq(0);
  });

  it("TEST 3: ----------- ADDING A WHITELISTER ROLE DONE CORRECTLY  ---------------", async () => {
    // Get The PDAs
    const [adminAccountPDA, adminAccountBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("admin"), lendanaAdmin.publicKey.toBuffer()],
        program.programId
      );

    const [globalTrustedPDA, globalTrustedBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("trusted_entities")],
        program.programId
      );

    // Call The Instruction
    await program.methods
      .grantWhitelister(whitelister.publicKey)
      .accounts({
        admin: lendanaAdmin.publicKey,
        //@ts-ignore
        adminAccount: adminAccountPDA,
        trustedRoles: globalTrustedPDA,
      })
      .signers([lendanaAdmin])
      .rpc();

    const trustedRolesData = await program.account.trustedEntities.fetch(
      globalTrustedPDA
    );
    expect(trustedRolesData.trustedRoles.length).to.eq(1);
  });

  it("TEST 5: ---------------- INITIALIZING A GLOBAL TOKEN WHITELIST REGISTRY AND POSITION COUNTERS ----------", async () => {
    // Getting The PDAs
    const [globalWhitelistedTokensPDA, globalWhitelistedTokensBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("all_whitelisted_tokens")],
        program.programId
      );

    const [lenderPositionCounterPDA, lenderPositionCounterBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("lenders_position_id_counter")],
        program.programId
      );

    const [borrowerPositionCounterPDA, borrowerPositionCounterBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("borrowers_position_id_counter")],
        program.programId
      );

    await program.methods
      .initWhitelistedRegistryAndCounters()
      .accounts({
        whitelisterRole: whitelister.publicKey,
      })
      .signers([whitelister])
      .rpc();

    const whitelistedRegistryData =
      await program.account.allWhitelistedTokens.fetch(
        globalWhitelistedTokensPDA
      );

    const lendersPositionsData =
      await program.account.lenderPositionIdCounter.fetch(
        lenderPositionCounterPDA
      );
    const borrowersPositionsData =
      await program.account.borrowerPositionIdCounter.fetch(
        borrowerPositionCounterPDA
      );

    expect(whitelistedRegistryData.tokensWhitelisted.length).to.eq(0);
    expect(lendersPositionsData.lendersCurrentPositionId.toNumber()).to.eq(0);
    expect(borrowersPositionsData.borrowersCurrentPositionId.toNumber()).to.eq(
      0
    );
  });

  it("TEST 6:  ----------- WHITELISTING A TOKEN ADDRESS ----------", async () => {
    // Get Global Token Registry
    const [globalWhitelistedTokensPDA, globalWhitelistedTokensBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("all_whitelisted_tokens")],
        program.programId
      );

    const [whitelisterPDA, whitelisterBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("whitelister"), whitelister.publicKey.toBuffer()],
      program.programId
    );

    // Token Mints
    // Let's Create The Mint token
    const usdcTokenMint = await createMint(
      provider.connection,
      whitelister,
      whitelister.publicKey,
      null,
      6
    );

    // Token Escrow Vault PDA
    const [usdcTokenVaultPDA, usdcTokenEscrowVaultBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("token_escrow"), usdcTokenMint.toBuffer()],
        program.programId
      );
    /* Let's create the Associated Token Mint Vault
    const tokenVault = await createAssociatedTokenAccount(
      provider.connection,
      whitelister,
      usdcToken.publicKey,
      usdcTokenVaultPDA
    );*/

    const tokenVaultAddress = getAssociatedTokenAddressSync(
      usdcTokenMint,
      usdcTokenVaultPDA,
      true,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    // Call The Instruction
    await program.methods
      .whitelistToken(usdcTokenMint)
      .accounts({
        whitelisterRole: whitelister.publicKey,
        //@ts-ignore
        allWhitelistedTokens: globalWhitelistedTokensPDA,
        whitelister: whitelisterPDA,
        tokenProgram: TOKEN_PROGRAM_ID,
        mintToken: usdcTokenMint,
        tokenVault: tokenVaultAddress,
        tokenEscrow: usdcTokenVaultPDA,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([whitelister])
      .rpc();

    // Verify usdcToken has been added to global whitelisted tokens
    const whitelistedRegistryData =
      await program.account.allWhitelistedTokens.fetch(
        globalWhitelistedTokensPDA
      );
    const tokenExists = whitelistedRegistryData.tokensWhitelisted.some(
      (tokenPublicKey) => tokenPublicKey.equals(usdcTokenMint)
    );
    expect(tokenExists).to.be.true;
    expect(whitelistedRegistryData.tokensWhitelisted.length).to.eq(1);

    // Query The Associated Token Escrow Vault creation
    const usdcTokenVaultPDAData = await program.account.lentTokenVault.fetch(
      usdcTokenVaultPDA
    );
    expect(usdcTokenVaultPDAData.lendingToken).deep.equal(usdcTokenMint);
    expect(usdcTokenVaultPDAData.totalLentTokens.toNumber()).to.eq(0);
    expect(usdcTokenVaultPDAData.isActive).to.be.true;
  });

  it("TEST 7:   ------------------- USER LENDS HIS TOKEN   ---------------------", async () => {
    /* + The User Got To Have Some Whitelisted Tokens already*/
    // Add this at the start of your test
    // Token Mints
    // Let's Create The Mint token
    const usdcTokenMint = await createMint(
      provider.connection,
      whitelister,
      whitelister.publicKey,
      null,
      6
    );

    /* Get Lender ATA, and mint Some usdcToken to Lender1
    const lender1ATAaddress = getAssociatedTokenAddressSync(
      usdcTokenMint,
      lender1.publicKey,
      true,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );*/
    const lender1ATAaddress = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      lender1,
      usdcTokenMint,
      lender1.publicKey
    );

    const userUsdcToken = await mintTo(
      provider.connection,
      lender1,
      usdcTokenMint,
      lender1ATAaddress.address,
      whitelister.publicKey,
      500 * 10 ** 6
    );

    // Get PDAs
    const [globalWhitelistedTokensPDA, globalWhitelistedTokensBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("all_whitelisted_tokens")],
        program.programId
      );

    const [lenderPositionCounterPDA, lenderPositionCounterBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("lenders_position_id_counter")],
        program.programId
      );

    const [tokenEscrowPDA, tokenEscrowBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("token_escrow"), usdcTokenMint.toBuffer()],
      program.programId
    );

    const tokenVaultAddress = getAssociatedTokenAddressSync(
      usdcTokenMint,
      tokenEscrowPDA,
      true,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const [lender1PositionPDA, lender1PositionBump] =
      PublicKey.findProgramAddressSync(
        [
          Buffer.from("lender_position"),
          lender1.publicKey.toBuffer(),
          usdcTokenMint.toBuffer(),
        ],
        program.programId
      );

    console.log("Lender1 PublicKey:", lender1.publicKey.toBase58());
    console.log("Token Escrow PDA:", tokenEscrowPDA.toBase58());
    console.log("Token Vault Address:", tokenVaultAddress.toBase58());
    console.log("Lender Position PDA:", lender1PositionPDA.toBase58());
    console.log(
      "Lender Position Counter PDA:",
      lenderPositionCounterPDA.toBase58()
    );

    // Call The Lend token Instruction
    const loanTerms = {
      interestRate: new BN(500),
      lendingDuration: new BN(7776000),
    };

    await program.methods
      .lendToken(new BN(450 * 10 ** 6), loanTerms)
      .accounts({
        lender: lender1.publicKey,
        tokenToLend: usdcTokenMint,
        //@ts-ignore
        lenderAta: lender1ATAaddress,
        allWhitelistedTokens: globalWhitelistedTokensPDA,
        tokenEscrow: tokenEscrowPDA,
        tokenVault: tokenVaultAddress,
        lenderPosition: lender1PositionPDA,
        lenderPositionIdCounter: lenderPositionCounterPDA,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([lender1])
      .rpc();
  });
});
