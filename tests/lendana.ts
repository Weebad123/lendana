import * as anchor from "@coral-xyz/anchor";
import { Program, Wallet } from "@coral-xyz/anchor";
import { Lendana } from "../target/types/lendana";
import {
  Connection,
  clusterApiUrl,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  NATIVE_MINT,
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
import { HermesClient } from "@pythnetwork/hermes-client";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";

describe("lendana", () => {
  // Configure the client to use the local cluster.

  /** ---------------------     TEST SETUP       -------------------- */
  /*const connection = new Connection("http://localhost:8899", {
    commitment: "confirmed",
    maxSupportedTransactionVersion: 0, // This parameter is required
  } as any);*/
  const provider = anchor.AnchorProvider.env();
  /*const provider = new anchor.AnchorProvider(
    connection,
    anchor.Wallet.local(),
    { preflightCommitment: "confirmed" }
  );*/
  anchor.setProvider(provider);

  const program = anchor.workspace.Lendana as Program<Lendana>;

  const TOKEN_PROGRAM: typeof TOKEN_2022_PROGRAM_ID | typeof TOKEN_PROGRAM_ID =
    TOKEN_2022_PROGRAM_ID;

  // Price Update Client
  const priceServiceConnection = new HermesClient(
    "https://hermes.pyth.network/",
    {}
  );

  // Actors In The System
  const deployer = provider.wallet;
  const lendanaAdmin = anchor.web3.Keypair.generate();
  const whitelister = anchor.web3.Keypair.generate();
  const lender1 = anchor.web3.Keypair.generate();
  const lender2 = anchor.web3.Keypair.generate();
  const borrower1 = anchor.web3.Keypair.generate();
  const borrower2 = anchor.web3.Keypair.generate();
  const borrower3 = anchor.web3.Keypair.generate();

  let connection: Connection;
  let wallet: Wallet;

  // Token Mints In our Testing
  let usdcTokenMint: PublicKey;
  let daiTokenMint: PublicKey;
  let ethTokenMint: PublicKey;
  const solMint = NATIVE_MINT;

  // Price Feed IDs
  // Define the constants for the price Feed ID hex
  const usdcTokenPriceFeedIdHex =
    "0xeaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a";
  const solMintPriceFeedIdHex =
    "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
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
      [
        lendanaAdmin.publicKey,
        whitelister.publicKey,
        lender1.publicKey,
        lender2.publicKey,
      ],
      5
    );

    await setupActors(
      provider,
      [borrower1.publicKey, borrower2.publicKey, borrower3.publicKey],
      50
    );

    // Create Token Mints
    usdcTokenMint = await createMint(
      provider.connection,
      whitelister,
      whitelister.publicKey,
      null,
      6
    );

    daiTokenMint = await createMint(
      provider.connection,
      whitelister,
      whitelister.publicKey,
      null,
      9
    );

    ethTokenMint = await createMint(
      provider.connection,
      whitelister,
      whitelister.publicKey,
      null,
      18
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

    const [priceFeedsRegistryPDA, priceFeedsRegistryBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("price_feed_registry")],
        program.programId
      );

    const [solCollateralVaultPDA, solCollateralVaultBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("sol_collateral_vault")],
        program.programId
      );

    await program.methods
      .initRegistriesAndCounters()
      .accounts({
        whitelisterRole: whitelister.publicKey,
        //@ts-ignore
        tokensPriceFeedRegistry: priceFeedsRegistryPDA,
        sol_collateral_vault: solCollateralVaultPDA,
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
    const priceFeedsRegistryData =
      await program.account.tokenPriceFeedRegistry.fetch(priceFeedsRegistryPDA);
    const solCollateralVaultData =
      await program.account.solCollateralVault.fetch(solCollateralVaultPDA);

    expect(whitelistedRegistryData.tokensWhitelisted.length).to.eq(0);
    expect(lendersPositionsData.lendersCurrentPositionId.toNumber()).to.eq(0);
    expect(borrowersPositionsData.borrowersCurrentPositionId.toNumber()).to.eq(
      0
    );
    expect(priceFeedsRegistryData.tokenPriceMapping.length).to.eq(0);
    expect(priceFeedsRegistryData.authority.toBuffer()).to.deep.equal(
      whitelister.publicKey.toBuffer()
    );
    expect(solCollateralVaultData.vaultBalance.toNumber()).to.eq(0);
    expect(solCollateralVaultData.isActive).to.be.true;
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

    // Token Escrow Vault PDA
    const [usdcTokenVaultPDA, usdcTokenEscrowVaultBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("token_escrow"), usdcTokenMint.toBuffer()],
        program.programId
      );

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
    const tokenVaultData = await getAccount(
      provider.connection,
      tokenVaultAddress
    );
    expect(tokenExists).to.be.true;
    expect(whitelistedRegistryData.tokensWhitelisted.length).to.eq(1);

    // Query The Associated Token Escrow Vault creation
    const usdcTokenVaultPDAData =
      await program.account.lentBorrowedTokenEscrow.fetch(usdcTokenVaultPDA);
    expect(usdcTokenVaultPDAData.lendingBorrowingToken).deep.equal(
      usdcTokenMint
    );
    expect(usdcTokenVaultPDAData.totalLentTokens.toNumber()).to.eq(0);
    expect(usdcTokenVaultPDAData.isActive).to.be.true;

    // Query The Associated Token Vault account balance to ensure it has no tokens
    expect(Number(tokenVaultData.amount)).to.eq(0);
  });

  it("TEST 7:   ------------------- LENDER1 AND LENDER2 LENDS THEIR WHITELISTED USDC TOKEN   ---------------------", async () => {
    /* + The User Got To Have Some Whitelisted Tokens already*/
    // Add this at the start of your test

    const lender1ATAaddress = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      lender1,
      usdcTokenMint,
      lender1.publicKey
    );

    const lenderUsdcToken = await mintTo(
      provider.connection,
      lender1,
      usdcTokenMint,
      lender1ATAaddress.address,
      whitelister.publicKey,
      500 * 10 ** 6,
      [whitelister]
    );

    const lender2ATAaddress = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      lender2,
      usdcTokenMint,
      lender2.publicKey
    );

    const lender2UsdcToken = await mintTo(
      provider.connection,
      lender2,
      usdcTokenMint,
      lender2ATAaddress.address,
      whitelister.publicKey,
      700 * 10 ** 6,
      [whitelister]
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

    const [lender2PositionPDA, lender2PositionBump] =
      PublicKey.findProgramAddressSync(
        [
          Buffer.from("lender_position"),
          lender2.publicKey.toBuffer(),
          usdcTokenMint.toBuffer(),
        ],
        program.programId
      );

    // Call The Lend token Instruction
    const lender1LoanTerms = {
      interestRate: new BN(500),
      lendingDuration: new BN(7776000),
    };

    const lender2LoanTerms = {
      interestRate: new BN(300),
      lendingDuration: new BN(2592000),
    };

    // Lender 1 Calls The Lend Token Instruction
    await program.methods
      .lendToken(new BN(450 * 10 ** 6), lender1LoanTerms)
      .accounts({
        lender: lender1.publicKey,
        tokenToLend: usdcTokenMint,
        //@ts-ignore
        lenderAta: lender1ATAaddress.address,
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

    // Lender 2 Calls The Lend Token Instruction
    await program.methods
      .lendToken(new BN(600 * 10 ** 6), lender2LoanTerms)
      .accounts({
        lender: lender2.publicKey,
        tokenToLend: usdcTokenMint,
        //@ts-ignore
        lenderAta: lender2ATAaddress.address,
        allWhitelistedTokens: globalWhitelistedTokensPDA,
        tokenEscrow: tokenEscrowPDA,
        tokenVault: tokenVaultAddress,
        lenderPosition: lender2PositionPDA,
        lenderPositionIdCounter: lenderPositionCounterPDA,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([lender2])
      .rpc();

    // Let's Get The PDA Data and Validate The On-chain data
    const lender1PositionData = await program.account.lenderPosition.fetch(
      lender1PositionPDA
    );
    const lender2PositionData = await program.account.lenderPosition.fetch(
      lender2PositionPDA
    );
    const tokenEscrowData = await program.account.lentBorrowedTokenEscrow.fetch(
      tokenEscrowPDA
    );
    const tokenVaultData = await getAccount(
      provider.connection,
      tokenVaultAddress
    );

    // Let's Verify The Lender1 Position
    expect(lender1PositionData.lenderPubkey).to.deep.equal(lender1.publicKey);
    expect(lender1PositionData.lendingToken).to.deep.equal(usdcTokenMint);
    expect(lender1PositionData.isPositionActive).to.be.true;
    expect(lender1PositionData.isMatched).to.be.false;
    expect(lender1PositionData.lenderPositionId.toNumber()).to.eq(1);
    expect(lender1PositionData.lendingAmount.toNumber()).to.eq(450 * 10 ** 6);
    expect(lender1PositionData.interestAccumulated.toNumber()).to.eq(0);
    expect(lender1PositionData.lendingTerms.interestRate.toNumber()).to.eq(500);
    expect(lender1PositionData.lendingTerms.lendingDuration.toNumber()).to.eq(
      7776000
    );

    // Let's Verify The Lender2 Position
    expect(lender2PositionData.lenderPubkey).to.deep.equal(lender2.publicKey);
    expect(lender2PositionData.lendingToken).to.deep.equal(usdcTokenMint);
    expect(lender2PositionData.isPositionActive).to.be.true;
    expect(lender2PositionData.isMatched).to.be.false;
    expect(lender2PositionData.lenderPositionId.toNumber()).to.eq(2);
    expect(lender2PositionData.lendingAmount.toNumber()).to.eq(600 * 10 ** 6);
    expect(lender2PositionData.interestAccumulated.toNumber()).to.eq(0);
    expect(lender2PositionData.lendingTerms.interestRate.toNumber()).to.eq(300);
    expect(lender2PositionData.lendingTerms.lendingDuration.toNumber()).to.eq(
      2592000
    );

    // Let's Verify Token Escrow Data
    expect(tokenEscrowData.isActive).to.be.true;
    expect(tokenEscrowData.totalLentTokens.toNumber()).to.eq(1050 * 10 ** 6);
    expect(tokenEscrowData.lendingBorrowingToken).to.deep.equal(usdcTokenMint);

    // Let's Ensure Token Vault Receives The Lent Tokens By Checking its Balance
    expect(Number(tokenVaultData.amount)).to.eq(1050 * 10 ** 6);
  });

  it("TEST 8:  UNHAPPY SCENARIO  ------------- LENDER2 TRIES TO LEND A NON-WHITELISTED TOKEN (daiTokenMint) SHOULD FAIL   ---------", async () => {
    const lender2ATAaddress = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      lender2,
      daiTokenMint,
      lender2.publicKey
    );

    const userDaiToken = await mintTo(
      provider.connection,
      lender2,
      daiTokenMint,
      lender2ATAaddress.address,
      whitelister.publicKey,
      1000 * 10 ** 9,
      [whitelister]
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
      [Buffer.from("token_escrow"), daiTokenMint.toBuffer()],
      program.programId
    );

    const tokenVaultAddress = getAssociatedTokenAddressSync(
      daiTokenMint,
      tokenEscrowPDA,
      true,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const [lender2PositionPDA, lender2PositionBump] =
      PublicKey.findProgramAddressSync(
        [
          Buffer.from("lender_position"),
          lender2.publicKey.toBuffer(),
          daiTokenMint.toBuffer(),
        ],
        program.programId
      );

    // Call The Lend token Instruction
    const loanTerms = {
      interestRate: new BN(700),
      lendingDuration: new BN(15552000),
    };

    try {
      await program.methods
        .lendToken(new BN(700 * 10 ** 9), loanTerms)
        .accounts({
          lender: lender2.publicKey,
          tokenToLend: daiTokenMint,
          //@ts-ignore
          lenderAta: lender2ATAaddress.address,
          allWhitelistedTokens: globalWhitelistedTokensPDA,
          tokenEscrow: tokenEscrowPDA,
          tokenVault: tokenVaultAddress,
          lenderPosition: lender2PositionPDA,
          lenderPositionIdCounter: lenderPositionCounterPDA,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([lender2])
        .rpc();
    } catch (err) {
      expect(err.error.errorCode.code).to.equal("AccountNotInitialized");
    }
  });

  //  -----------------                MODIFYING LENDING POSITION TESTS              ---------------------
  it("TEST 9: ------------  LENDER1 TRIES TO MODIFY HIS LENDING POSITION  --------------", async () => {
    // Get Required Accounts
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

    const lender1ATAaddress = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      lender1,
      usdcTokenMint,
      lender1.publicKey
    );

    // New Loan Terms
    const newLoanTerms = {
      interestRate: new BN(700),
      lendingDuration: new BN(15552000),
    };

    // Let's Call The Modify Lender Position instruction
    await program.methods
      .modifyLenderPosition(newLoanTerms, new BN(35 * 10 ** 6))
      .accounts({
        lender: lender1.publicKey,
        tokenToLend: usdcTokenMint,
        //@ts-ignore
        lenderAta: lender1ATAaddress.address,
        tokenEscrow: tokenEscrowPDA,
        tokenVault: tokenVaultAddress,
        lenderPosition: lender1PositionPDA,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([lender1])
      .rpc();

    // Let's Get The PDA Data and Validate The On-chain data
    const lenderPositionData = await program.account.lenderPosition.fetch(
      lender1PositionPDA
    );
    const tokenEscrowData = await program.account.lentBorrowedTokenEscrow.fetch(
      tokenEscrowPDA
    );
    const tokenVaultData = await getAccount(
      provider.connection,
      tokenVaultAddress
    );

    // Let's Verify The Lender Position To Confirm If Changes Were Effected
    expect(lenderPositionData.lenderPubkey).to.deep.equal(lender1.publicKey);
    expect(lenderPositionData.lendingToken).to.deep.equal(usdcTokenMint);
    expect(lenderPositionData.isPositionActive).to.be.true;
    expect(lenderPositionData.isMatched).to.be.false;
    expect(lenderPositionData.lenderPositionId.toNumber()).to.eq(1);
    expect(lenderPositionData.lendingAmount.toNumber()).to.eq(485 * 10 ** 6);
    expect(lenderPositionData.interestAccumulated.toNumber()).to.eq(0);
    expect(lenderPositionData.lendingTerms.interestRate.toNumber()).to.eq(700);
    expect(lenderPositionData.lendingTerms.lendingDuration.toNumber()).to.eq(
      15552000
    );

    // Let's Verify Token Escrow Data
    expect(tokenEscrowData.isActive).to.be.true;
    expect(tokenEscrowData.totalLentTokens.toNumber()).to.eq(1085 * 10 ** 6);
    expect(tokenEscrowData.lendingBorrowingToken).to.deep.equal(usdcTokenMint);

    // Let's Ensure Token Vault Receives The Lent Tokens By Checking its Balance
    expect(Number(tokenVaultData.amount)).to.eq(1085 * 10 ** 6);
  });

  // -------------------- CANCEL A LENDING ORDER TESTINGS -------------------------------
  it("TEST 10: -------------- LENDER2 CANCELS HIS LENDING ORDER  --------------", async () => {
    // Get Required Accounts
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

    const lender2ATAaddress = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      lender2,
      usdcTokenMint,
      lender2.publicKey
    );
    // Let's Check Lender2 balance prior to cancelling the order
    const lender2ATADataBeforeCancel = await getAccount(
      provider.connection,
      lender2ATAaddress.address
    );
    expect(Number(lender2ATADataBeforeCancel.amount)).to.eq(100 * 10 ** 6);

    const [lender2PositionPDA, lender2PositionBump] =
      PublicKey.findProgramAddressSync(
        [
          Buffer.from("lender_position"),
          lender2.publicKey.toBuffer(),
          usdcTokenMint.toBuffer(),
        ],
        program.programId
      );

    // Let's call the Cancel Order Instruction
    await program.methods
      .cancelLendingOrder()
      .accounts({
        lender: lender2.publicKey,
        tokenToLend: usdcTokenMint,
        //@ts-ignore
        lenderAta: lender2ATAaddress.address,
        tokenEscrow: tokenEscrowPDA,
        tokenVault: tokenVaultAddress,
        lenderPosition: lender2PositionPDA,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([lender2])
      .rpc();

    // Let's Make Assertions and Validations

    const tokenEscrowData = await program.account.lentBorrowedTokenEscrow.fetch(
      tokenEscrowPDA
    );
    const tokenVaultData = await getAccount(
      provider.connection,
      tokenVaultAddress
    );
    // Check if Lender2 was refunded his lent token
    const lender2ATADataAfterCancel = await getAccount(
      provider.connection,
      lender2ATAaddress.address
    );
    expect(Number(lender2ATADataAfterCancel.amount)).to.eq(700 * 10 ** 6);

    // Check if TokenEscrow updates the total lent tokens after a cancelling order
    expect(tokenEscrowData.totalLentTokens.toNumber()).to.eq(485 * 10 ** 6);

    // Check If Token Vault Balance Was Reduced After Cancelling Order
    expect(Number(tokenVaultData.amount)).to.eq(485 * 10 ** 6);

    // Now, i will use the getAccountInfo function on the lender2PositionPDA pda, and if it's indeed close,
    // solana runtime will return a null
    const lender2PositionData = await provider.connection.getAccountInfo(
      lender2PositionPDA
    );
    expect(lender2PositionData).to.eq(null);
  });

  // -------------------    ADD A PRICE FEED ID FOR A WHITELISTED TOKEN      -------------------------
  it("TEST 11:  ----------------- ADDING A PRICE FEED ID FOR USDC WHITELISTED TOKEN  ------------------", async () => {
    const [priceFeedsRegistryPDA, priceFeedsRegistryBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("price_feed_registry")],
        program.programId
      );
    // Call The Actual Instruction
    await program.methods
      .addPrice(usdcTokenMint, usdcTokenPriceFeedIdHex)
      .accounts({
        whitelister: whitelister.publicKey,
        //@ts-ignore
        tokensPriceFeedsRegistry: priceFeedsRegistryPDA,
      })
      .signers([whitelister])
      .rpc();

    // Add the solMint too
    await program.methods
      .addPrice(solMint, solMintPriceFeedIdHex)
      .accounts({
        whitelister: whitelister.publicKey,
        //@ts-ignore
        tokensPriceFeedRegistry: priceFeedsRegistryPDA,
      })
      .signers([whitelister])
      .rpc();

    // Get The Price Feeds Registry Data
    const priceFeedsRegistryData =
      await program.account.tokenPriceFeedRegistry.fetch(priceFeedsRegistryPDA);

    expect(priceFeedsRegistryData.tokenPriceMapping.length).to.eq(2);
    expect(priceFeedsRegistryData.tokenPriceMapping[0].tokenMint).to.deep.equal(
      usdcTokenMint
    );
    expect(
      priceFeedsRegistryData.tokenPriceMapping[0].priceFeedId.toString()
    ).to.equal(usdcTokenPriceFeedIdHex);
    expect(priceFeedsRegistryData.tokenPriceMapping[1].tokenMint).to.deep.equal(
      solMint
    );
    expect(
      priceFeedsRegistryData.tokenPriceMapping[1].priceFeedId.toString()
    ).to.equal(solMintPriceFeedIdHex);
  });

  // -----------------     BORROWING A TOKEN        ------------------------
  it("TEST 12:  -----------------------  BORROWING A WHITELISTED USDC TOKEN WITH NATIVE SOL AS COLLATERAL   ---------", async () => {
    // Get Required PDAs and Accounts
    // Borrower Need To Have ATA for USDC token
    const borrower1UsdcATAaddress = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      borrower1,
      usdcTokenMint,
      borrower1.publicKey
    );

    const [globalWhitelistedTokensPDA, globalWhitelistedTokensBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("all_whitelisted_tokens")],
        program.programId
      );

    const [BorrowerPositionCounterPDA, BorrowerPositionCounterBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("borrowers_position_id_counter")],
        program.programId
      );

    const [borrowingTokenEscrowPDA, borrowingTokenEscrowBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("token_escrow"), usdcTokenMint.toBuffer()],
        program.programId
      );

    const usdcTokenVaultAddress = getAssociatedTokenAddressSync(
      usdcTokenMint,
      borrowingTokenEscrowPDA,
      true,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const [borrower1PositionPDA, borrower1PositionBump] =
      PublicKey.findProgramAddressSync(
        [
          Buffer.from("borrower_position"),
          borrower1.publicKey.toBuffer(),
          usdcTokenMint.toBuffer(),
        ],
        program.programId
      );

    const [priceFeedsRegistryPDA, priceFeedsRegistryBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("price_feed_registry")],
        program.programId
      );

    const [solCollateralVaultPDA, solCollateralVaultBump] =
      PublicKey.findProgramAddressSync(
        [Buffer.from("sol_collateral_vault")],
        program.programId
      );

    // Call The Borrow Token Instruction
    const borrowingLoanTerms = {
      interestRate: new BN(700),
      lendingDuration: new BN(15552000),
    };

    //Collateral And Borrowing Token Price Updates
    const borrowingPriceUpdate = (
      await priceServiceConnection.getLatestPriceUpdates(
        [usdcTokenPriceFeedIdHex],
        { encoding: "base64" }
      )
    ).binary.data;

    const collateralPriceUpdate = (
      await priceServiceConnection.getLatestPriceUpdates(
        [solMintPriceFeedIdHex],
        { encoding: "base64" }
      )
    ).binary.data;

    // Post Price Update To Solana
    const pythSolanaReceiver = new PythSolanaReceiver({
      connection: provider.connection,

      wallet: provider.wallet as any,
    });
    // Create transaction builder
    const transactionBuilder = pythSolanaReceiver.newTransactionBuilder({
      closeUpdateAccounts: false,
    });
    // Add Both Price Updates To The Builder
    await transactionBuilder.addPostPriceUpdates([
      ...borrowingPriceUpdate,
      ...collateralPriceUpdate,
    ]);

    // Add Borrow instruction to the same transaction
    await transactionBuilder.addPriceConsumerInstructions(
      async (getPriceUpdateAccount) => {
        const borrowingPriceAccount = getPriceUpdateAccount(
          usdcTokenPriceFeedIdHex
        );
        const collateralPriceAccount = getPriceUpdateAccount(
          solMintPriceFeedIdHex
        );

        // Create your instruction with these accounts
        const borrowTokenIx = await program.methods
          .borrowToken(solMint, usdcTokenMint, new BN(200), borrowingLoanTerms)
          .accounts({
            borrower: borrower1.publicKey,
            tokenToBorrow: usdcTokenMint,
            tokenCollateral: solMint,
            collateralPriceUpdate: collateralPriceAccount,
            borrowingPriceUpdate: borrowingPriceAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            //@ts-ignore
            systemProgram: SystemProgram.programId,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          })
          .signers([borrower1])
          .instruction();

        return [
          {
            instruction: borrowTokenIx,
            signers: [borrower1],
          },
        ];
      }
    );
    // Send the transactions
    await pythSolanaReceiver.provider.sendAll(
      await transactionBuilder.buildVersionedTransactions({
        computeUnitPriceMicroLamports: 5000,
      }),
      { skipPreflight: true }
    );
  });
});
