import * as anchor from "@coral-xyz/anchor";
import { Program, BN, web3  } from "@coral-xyz/anchor";
import { FtrxLending } from "../target/types/ftrx_lending";
import {
  getAccount,
  getOrCreateAssociatedTokenAccount,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
  createSyncNativeInstruction,
  createCloseAccountInstruction,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  NATIVE_MINT,
  getMint,
} from "@solana/spl-token";


import {
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
} from "@solana/web3.js";
import { superUserKey, testUserKey } from "./testKeys";




describe("ftrx_lending", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.FtrxLending as Program<FtrxLending>;
  const connection = program.provider.connection;
  const superUser = superUserKey.keypair;
  const testUser = testUserKey.keypair;
  console.log(superUser.publicKey)
  console.log(testUser.publicKey)
  const testUserWallet = new anchor.Wallet(testUser);

  let usdcMint=new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
  let wsolMint=new PublicKey("So11111111111111111111111111111111111111112");
  
  function wsol_account_creation_instruction(owner,associatedToken){
      

    const transaction = new Transaction().add(
        createAssociatedTokenAccountInstruction(
            owner.publicKey,
            associatedToken,
            owner.publicKey,
            NATIVE_MINT,
            TOKEN_PROGRAM_ID,
            ASSOCIATED_TOKEN_PROGRAM_ID,
        ),
        createSyncNativeInstruction(associatedToken, TOKEN_PROGRAM_ID),
    );

    return transaction
    
  }


  let [simplePoolKey, simplePoolKeyBump] =
  web3.PublicKey.findProgramAddressSync(
    [
      usdcMint.toBuffer(),
      wsolMint.toBuffer(),
      superUser.publicKey.toBuffer(),
    ],
    program.programId
  );


  let [simpleUserAccountKey, simpleUserAccountBump] =
  web3.PublicKey.findProgramAddressSync(
    [
      simplePoolKey.toBuffer(),
      superUser.publicKey.toBuffer(),
    ],
    program.programId
  );

  
  let [secondSimpleUserAccountKey, secondSimpleUserAccountBump] =
  web3.PublicKey.findProgramAddressSync(
    [
      simplePoolKey.toBuffer(),
      testUser.publicKey.toBuffer(),
    ],
    program.programId
  );

  let str_disc_share="share"

  let [volatileShareMintKey, volatileShareMintKeyBump] =
  web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(str_disc_share),
      wsolMint.toBuffer(),
      simplePoolKey.toBuffer(),
    ],
    program.programId
  );


  let [stableShareMintKey, stableShareMintBump] =
  web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(str_disc_share),
      usdcMint.toBuffer(),
      simplePoolKey.toBuffer(),
    ],
    program.programId
  );

  let str_disc_vault="vault"


  let [volatileVaultKey, volatileVaultBump] =
  web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(str_disc_vault),
      wsolMint.toBuffer(),
      simplePoolKey.toBuffer(),
    ],
    program.programId
  );

  let [stableVaultKey, stableVaultBump] =
  web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from(str_disc_vault),
      usdcMint.toBuffer(),
      simplePoolKey.toBuffer(),
    ],
    program.programId
  );

  const pythFeed = new PublicKey("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");

  
  let accounts: any = {
    
    simplePool:simplePoolKey,
    poolAdmin:superUser.publicKey,
    userAdmin:superUser.publicKey,
    userToLiquidate:secondSimpleUserAccountKey,
    userState:simpleUserAccountKey,
    volatileShareMint:volatileShareMintKey,
    stableShareMint:stableShareMintKey,
    stableVault:stableVaultKey,
    volatileVault:volatileVaultKey,
    stableMint:usdcMint,
    volatileMint:wsolMint,
    pythFeed,
    systemProgram: SystemProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    rent: SYSVAR_RENT_PUBKEY,
  };

  let accounts_second_user: any = {
    
    simplePool:simplePoolKey,
    userSigner:testUser.publicKey,
    userAuthority:testUser.publicKey,
    signer:testUser.publicKey,
    userVolatileVault:testUser.publicKey,
    userState:secondSimpleUserAccountKey,
    userToLiquidate:simpleUserAccountKey,
    volatileShareMint:volatileShareMintKey,
    stableShareMint:stableShareMintKey,
    stableVault:stableVaultKey,
    volatileVault:volatileVaultKey,
    stableMint:usdcMint,
    volatileMint:wsolMint,
    pythFeed,
    systemProgram: SystemProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    rent: SYSVAR_RENT_PUBKEY,
  };

  let borrower_borrows_volatile=true
  it("Is initialized!", async () => {
    const testUserCollateralAta = new PublicKey("51um5dCXFCtF55oju2rFoWezQkaPG2ZFRmeffbkdRSU2")

    const userCollateralAta = new PublicKey("6fM785rhRecw5X7tSshk7sQmUYphqBGkCuA3HkoEJLKt")

    accounts.userStableVault=userCollateralAta
    accounts.userVolatileVault=superUser.publicKey
    accounts_second_user.userVolatileVault=testUser.publicKey
    accounts_second_user.userStableVault=testUserCollateralAta

    const txHash1 = await connection.requestAirdrop(
      superUser.publicKey,
      LAMPORTS_PER_SOL * 10000
    );
    const txHash2 = await connection.requestAirdrop(
      testUser.publicKey,
      LAMPORTS_PER_SOL * 10000
    );
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });

  it("Create lending pool", async () => {
    // Add your test here.
    const target_utilization_in = new BN(800_000_000_000);//=101
    const protocol_fee_in       = new BN(3_000_000_000);//=30%
    const insurance_fund_fee_in = new BN(3_000_000_000);//=1%
    const new_borrow_ltv        = new BN(800_000);//=1%
    const liquidation_ltv       = new BN(880_000);//=1%
    const decimal_wsol=new BN(9);
    const decimal_usdc=new BN(6);

      
    const tx = await program.methods.adminCreatesSp(simplePoolKeyBump,target_utilization_in,protocol_fee_in,insurance_fund_fee_in,new_borrow_ltv,liquidation_ltv,decimal_wsol,decimal_usdc).accounts(accounts).signers([superUser]).rpc();
    console.log("Your transaction signature", tx);
  });


  it("Create user account", async () => {
    // Add your test here.
    const tx = await program.methods.suserCreatesUa(simpleUserAccountBump).accounts(accounts).signers([superUser]).rpc();
    console.log("Your transaction signature", tx);
  });

  it("Deposit sol & USDC", async () => {
    // Add your test here.
    const asset_index = new BN(0);//=101
    const asset_amount_usdc = new BN(1_000_000);//=30%
    const asset_amount_sol = new BN(300_000_000);//=30%
    
    let volatile_before = await getAccount(connection, accounts.volatileVault);
    let stable_before = await getAccount(connection, accounts.stableVault);
    const tx1 = await program.methods.suserDeposits(0,asset_amount_usdc).accounts(accounts).signers([superUser]).rpc();
    const tx2 = await program.methods.suserDeposits(1,asset_amount_sol).accounts(accounts).signers([superUser]).rpc();
    console.log("Your transaction signature", tx1);
    console.log("Your transaction signature", tx2);
    let volatile_after = await getAccount(connection, accounts.volatileVault);
    let stable_after = await getAccount(connection, accounts.stableVault);
    let tokenAccountTreas_before = await getAccount(connection, accounts.volatileVault);
    console.log("Volatile before",Number(volatile_before.amount))
    console.log("Volatile after",Number(volatile_after.amount))
    console.log("Stable before",Number(stable_before.amount))
    console.log("Stable after",Number(stable_after.amount))

    const txDetails = await program.provider.connection.getTransaction(tx2, {
      maxSupportedTransactionVersion: 0,
      commitment: "confirmed",
    });
    console.log("DEPOSIT DETAILS",txDetails.meta.logMessages)

    const txDetails2 = await program.provider.connection.getTransaction(tx1, {
      maxSupportedTransactionVersion: 0,
      commitment: "confirmed",
    });
    console.log("DEPOSIT DETAILS",txDetails2.meta.logMessages)

    const simplePool = await program.account.simplePool.fetch(
      accounts.simplePool
    );
    console.log(simplePool)

  });

  it("Create borrower user account", async () => {
    // Add your test here.
 
    const tx = await program.methods.suserCreatesUa(secondSimpleUserAccountBump).accounts(accounts_second_user).signers([testUser]).rpc();
    console.log("Your transaction signature", tx);
  });


  it("Borrower deposits USDC", async () => {
    // Add your test here.
    if (borrower_borrows_volatile){
    const asset_index = new BN(0);//=101
    const asset_amount = new BN(5_000_000);//=30%
    const tx1 = await program.methods.suserDeposits(0,asset_amount).accounts(accounts_second_user).signers([testUser]).rpc();
    console.log("Your transaction signature", tx1);



    const { lastValidBlockHeight, blockhash } =
    await connection.getLatestBlockhash();

    let output_tx=await connection.confirmTransaction(
      {
        blockhash: blockhash,
        lastValidBlockHeight: lastValidBlockHeight,
        signature: tx1,
      },
      "confirmed",
    );


    const txDetails2 = await program.provider.connection.getTransaction(tx1, {
      maxSupportedTransactionVersion: 0,
      commitment: "confirmed",
    });
    console.log("DEPOSIT DETAILS BORROWER ",txDetails2.meta.logMessages)

  }else{
    const asset_index = new BN(1);//=101
    const asset_amount = new BN(10_000_000);//=30%
    const tx1 = await program.methods.suserDeposits(1,asset_amount).accounts(accounts_second_user).signers([testUser]).rpc();
    console.log("Your transaction signature", tx1);
  }

  

  });

  it("Borrower borrows Volatile", async () => {
    // Add your test here.
    if (borrower_borrows_volatile){
    const asset_index = new BN(0);//=101
    const asset_amount = new BN(25_000_000);//=30%

    const associatedToken = getAssociatedTokenAddressSync(
        NATIVE_MINT,
        testUser.publicKey,
        false,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    accounts_second_user.userVolatileVault=associatedToken
    
    let initial_instruction=wsol_account_creation_instruction(testUser,associatedToken)
    let second_instruction= await program.methods.suserBorrows(1,asset_amount).accounts(accounts_second_user).signers([testUser]).instruction();
    initial_instruction.add(second_instruction)
    initial_instruction.add(
      createCloseAccountInstruction(associatedToken, testUser.publicKey, testUser.publicKey)
      
    );

    let tx1=  await connection.sendTransaction(initial_instruction, [testUser]);
    

    console.log("Your transaction signature", tx1);
    
    const { lastValidBlockHeight, blockhash } =
    await connection.getLatestBlockhash();

    let output_tx=await connection.confirmTransaction(
      {
        blockhash: blockhash,
        lastValidBlockHeight: lastValidBlockHeight,
        signature: tx1,
      },
      "confirmed",
    );
    const simplePool = await program.account.simplePool.fetch(
      accounts.simplePool
    );

    const txDetails = await program.provider.connection.getTransaction(tx1, {
      maxSupportedTransactionVersion: 0,
      commitment: "confirmed",
    });
    console.log("BORROWING DETAILS",txDetails.meta.logMessages)
  }else{
    const asset_index = new BN(0);//=101
    const asset_amount = new BN(1_000_000);//=30%

    const associatedToken = getAssociatedTokenAddressSync(
        NATIVE_MINT,
        testUser.publicKey,
        false,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    accounts_second_user.userVolatileVault=associatedToken
    
    let initial_instruction=wsol_account_creation_instruction(testUser,associatedToken)
    let second_instruction= await program.methods.suserBorrows(0,asset_amount).accounts(accounts_second_user).signers([testUser]).instruction();
    initial_instruction.add(second_instruction)
    initial_instruction.add(
      createCloseAccountInstruction(associatedToken, testUser.publicKey, testUser.publicKey)
      
    );

    let tx1=  await connection.sendTransaction(initial_instruction, [testUser]);
    

    console.log("Your transaction signature", tx1);
    
    const { lastValidBlockHeight, blockhash } =
    await connection.getLatestBlockhash();

    let output_tx=await connection.confirmTransaction(
      {
        blockhash: blockhash,
        lastValidBlockHeight: lastValidBlockHeight,
        signature: tx1,
      },
      "confirmed",
    );
    const simplePool = await program.account.simplePool.fetch(
      accounts.simplePool
    );

    const txDetails = await program.provider.connection.getTransaction(tx1, {
      maxSupportedTransactionVersion: 0,
      commitment: "confirmed",
    });
    console.log("BORROWING DETAILS",txDetails.meta.logMessages)

  }

  });



  it("Borrower gets liquidated", async () => {
    // Add your test here.
    if (borrower_borrows_volatile){

      const asset_index = new BN(1);//=101
      const asset_amount = new BN(20_000);//=30%
      accounts.userToLiquidateState=secondSimpleUserAccountKey
      const tx1 = await program.methods.suserLiquidates(1,asset_amount).accounts(accounts).signers([superUser]).rpc();
      console.log("Your transaction signature", tx1);
      console.log("Your transaction signature", tx1);


      const { lastValidBlockHeight, blockhash } =
      await connection.getLatestBlockhash();
  
      let output_tx=await connection.confirmTransaction(
        {
          blockhash: blockhash,
          lastValidBlockHeight: lastValidBlockHeight,
          signature: tx1,
        },
        "confirmed",
      );
      const simplePool = await program.account.simplePool.fetch(
        accounts.simplePool
      );

      const txDetails = await program.provider.connection.getTransaction(tx1, {
        maxSupportedTransactionVersion: 0,
        commitment: "confirmed",
      });
      console.log("LIQUIDATION DETAILS",txDetails.meta.logMessages)

      

    }else{

      const asset_index = new BN(1);//=101
      const asset_amount = new BN(1_000_000);//=30%
      accounts.userToLiquidateState=secondSimpleUserAccountKey
      const tx1 = await program.methods.suserLiquidates(0,asset_amount).accounts(accounts).signers([superUser]).rpc();
      console.log("Your transaction signature", tx1);


      const { lastValidBlockHeight, blockhash } =
      await connection.getLatestBlockhash();
  
    let output_tx=await connection.confirmTransaction(
      {
        blockhash: blockhash,
        lastValidBlockHeight: lastValidBlockHeight,
        signature: tx1,
      },
      "confirmed",
    );
    const simplePool = await program.account.simplePool.fetch(
      accounts.simplePool
    );

    const txDetails = await program.provider.connection.getTransaction(tx1, {
      maxSupportedTransactionVersion: 0,
      commitment: "confirmed",
    });
    console.log("LIQUIDATION DETAILS",txDetails.meta.logMessages)

    
  
    }
      });

  it("Borrower redeeems Volatile", async () => {
    // Add your test here.
    if (borrower_borrows_volatile){
    const asset_index = new BN(0);//=101
    const asset_amount = new BN(25_000_000);//=30%

    const associatedToken = getAssociatedTokenAddressSync(
        NATIVE_MINT,
        testUser.publicKey,
        false,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    accounts_second_user.userVolatileVault=associatedToken
    
    let initial_instruction=wsol_account_creation_instruction(testUser,associatedToken)
    let second_instruction= await program.methods.suserRedeems(1,asset_amount).accounts(accounts_second_user).signers([testUser]).instruction();
    initial_instruction.add(second_instruction)
    initial_instruction.add(
      createCloseAccountInstruction(associatedToken, testUser.publicKey, testUser.publicKey)
      
    );

    let tx1=  await connection.sendTransaction(initial_instruction, [testUser]);
    

    console.log("Your transaction signature", tx1);
    
    const { lastValidBlockHeight, blockhash } =
    await connection.getLatestBlockhash();

    let output_tx=await connection.confirmTransaction(
      {
        blockhash: blockhash,
        lastValidBlockHeight: lastValidBlockHeight,
        signature: tx1,
      },
      "confirmed",
    );
    const simplePool = await program.account.simplePool.fetch(
      accounts.simplePool
    );

    const txDetails = await program.provider.connection.getTransaction(tx1, {
      maxSupportedTransactionVersion: 0,
      commitment: "confirmed",
    });
    console.log("BORROWING DETAILS",txDetails.meta.logMessages)
  }else{
    const asset_index = new BN(0);//=101
    const asset_amount = new BN(1_000_000);//=30%

    const associatedToken = getAssociatedTokenAddressSync(
        NATIVE_MINT,
        testUser.publicKey,
        false,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    accounts_second_user.userVolatileVault=associatedToken
    
    let initial_instruction=wsol_account_creation_instruction(testUser,associatedToken)
    let second_instruction= await program.methods.suserRedeems(0,asset_amount).accounts(accounts_second_user).signers([testUser]).instruction();
    initial_instruction.add(second_instruction)
    initial_instruction.add(
      createCloseAccountInstruction(associatedToken, testUser.publicKey, testUser.publicKey)
      
    );

    let tx1=  await connection.sendTransaction(initial_instruction, [testUser]);
    

    console.log("Your transaction signature", tx1);
    
    const { lastValidBlockHeight, blockhash } =
    await connection.getLatestBlockhash();

    let output_tx=await connection.confirmTransaction(
      {
        blockhash: blockhash,
        lastValidBlockHeight: lastValidBlockHeight,
        signature: tx1,
      },
      "confirmed",
    );
    const simplePool = await program.account.simplePool.fetch(
      accounts.simplePool
    );

    const txDetails = await program.provider.connection.getTransaction(tx1, {
      maxSupportedTransactionVersion: 0,
      commitment: "confirmed",
    });
    console.log("REDEEMING DETAILS",txDetails.meta.logMessages)

  }

  });





  it("Admin liquidates borrower", async () => {
    // Add your test here.
    const asset_index = new BN(0);//=101
    const asset_amount = new BN(0);//=30%
    const tx1 = await program.methods.adminLiquidatesSp(0,asset_amount).accounts(accounts).signers([superUser]).rpc();
    console.log("Your transaction signature", tx1);
  });


  it("Admin liquidates borrower at loss", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });

  it("Admin withnesses bad debt generation", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });

  it("User withdraws", async () => {
    // Add your test here.
    const asset_index = new BN(0);//=101
    const asset_amount = new BN(1);//=30%

    let volatile_before = await getAccount(connection, accounts.volatileVault);
    let stable_before = await getAccount(connection, accounts.stableVault);

    const associatedToken = getAssociatedTokenAddressSync(
        NATIVE_MINT,
        superUser.publicKey,
        false,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    accounts.userVolatileVault=associatedToken
    
    let initial_instruction=wsol_account_creation_instruction(superUser,associatedToken)
    
    let specific_instruction =await program.methods.suserWithdraws(1,asset_amount).accounts(accounts).signers([superUser]).instruction()
    let specific_instruction2 =await program.methods.suserWithdraws(0,asset_amount).accounts(accounts).signers([superUser]).instruction()
    initial_instruction.add(specific_instruction)
    initial_instruction.add(specific_instruction2)
    initial_instruction.add(
      createCloseAccountInstruction(associatedToken, superUser.publicKey, superUser.publicKey)
      
    );

    let tx1=  await connection.sendTransaction(initial_instruction, [superUser]);
    //let tx2=  await program.methods.suserWithdraws(0,asset_amount).accounts(accounts).signers([superUser]).rpc()
    const { lastValidBlockHeight, blockhash } =
    await connection.getLatestBlockhash();

    await connection.confirmTransaction(
      {
        blockhash: blockhash,
        lastValidBlockHeight: lastValidBlockHeight,
        signature: tx1,
      },
      "confirmed",
    );

    //const tx11 = await program.methods.suserWithdraws(0,asset_amount).accounts(accounts).signers([superUser]).rpc();
    //const tx2 = await program.methods.suserWithdraws(1,asset_amount).accounts(accounts).signers([superUser]).rpc();
    
    let volatile_after = await getAccount(connection, accounts.volatileVault);
    let stable_after = await getAccount(connection, accounts.stableVault);
    
    console.log("Your transaction signature", tx1);
    //console.log("Your transaction signature", tx11);
    //console.log("Your transaction signature", tx2);
    console.log("Volatile before",Number(volatile_before.amount))
    console.log("Volatile after",Number(volatile_after.amount))
    console.log("Stable before",Number(stable_before.amount))
    console.log("Stable after",Number(stable_after.amount))



    
  });


});
