import * as anchor from '@coral-xyz/anchor'
import { BN, Program,  } from '@coral-xyz/anchor'
import NodeWallet from '@coral-xyz/anchor/dist/cjs/nodewallet';
import { PublicKey, Keypair, } from '@solana/web3.js'
import { ProgramTestContext, startAnchor, BanksClient, Clock } from 'solana-bankrun'
import { SYSVAR_CLOCK_PUBKEY } from '@solana/web3.js'
import {createMint, mintTo } from 'spl-token-bankrun'

import IDL from '../target/idl/tokenvesting.json'
import { Tokenvesting} from '../target/types/tokenvesting'
import { SYSTEM_PROGRAM_ID,   } from '@coral-xyz/anchor/dist/cjs/native/system'
import { BankrunProvider,  } from 'anchor-bankrun'
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { publicKey } from '@coral-xyz/anchor/dist/cjs/utils';


describe('Vesting Smart Contract Tests', () => {
  const company = 'company'
  let beneficiary: Keypair;
  let context: ProgramTestContext;
  let provider: BankrunProvider;
  let program: Program<Tokenvesting>;
  let banksClient: BanksClient;
  let employer: Keypair;
  let mint: PublicKey;
  let beneficiaryProvider: BankrunProvider;
  let program2: Program<Tokenvesting>;
  let tokenvestingAccountKey: PublicKey;
  let treasuryTokenAccount: PublicKey;
  let employeeAccount: PublicKey;
  
  



  beforeAll(async () => {
    beneficiary = new anchor.web3.Keypair();

    context = await startAnchor(
      '',
      [{name: 'tokenvesting', programId: new PublicKey(IDL.address)}],
      [
        {
          address: beneficiary.publicKey,
          info: {
            lamports: 1_000_000_000,
            data: Buffer.alloc(0),
            owner: SYSTEM_PROGRAM_ID,
            executable: false,
          },
        },
      ]
    );

    provider = new BankrunProvider(context)
    anchor.setProvider(provider);
    program = new Program<Tokenvesting>(IDL as Tokenvesting, provider);
    banksClient = context.banksClient;
    employer = provider.wallet.payer;

    //@ts-expect-error - Type error in spl-token-bankrun dependency
    mint = await createMint(banksClient, employer, employer.publicKey, null, 2);

    beneficiaryProvider = new BankrunProvider(context);
    beneficiaryProvider.wallet = new NodeWallet(beneficiary);

    program2 = new Program<Tokenvesting>(IDL as Tokenvesting, beneficiaryProvider);
   
    [tokenvestingAccountKey] = PublicKey.findProgramAddressSync(
      [Buffer.from("company")],
      program.programId
    );
   
    [treasuryTokenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("tokenvesting_treasury"),Buffer.from("company") ],
      program.programId
    );
    [employeeAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("employee_tokenvesting"), 
        beneficiary.publicKey.toBuffer(), 
        tokenvestingAccountKey.toBuffer()],
      program.programId
    );
  });  
  it("should create a vesting account", async () => {
    const tx = await program.methods
    .createVestingAccount("company")
    .accounts({
      signer: employer.publicKey,
      mint,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc({commitment: 'confirmed'});
    
    const vestingAccountData = await program.account.tokenvestingAccount
    .fetch(tokenvestingAccountKey, 'confirmed');
  
    console.log('Vesting account data:', vestingAccountData, null, 2);
    console.log("Create vesting account:", tx);
  
  });
  it ("should fund the treasury token account", async () => {
    // const vestingAccountData = await program.account.tokenvestingAccount.fetch(tokenvestingAccountKey, 'confirmed');
    // const treasuryBump = vestingAccountData.treasuryBump;
    // [treasuryTokenAccount] = PublicKey.findProgramAddressSync(
    //   [
    //     Buffer.from("tokenvesting_treasury"), 
    //     Buffer.from(company),
    //     Buffer.from([treasuryBump]) // Add bump here
    //   ],
    //   program.programId
    // );
    const amount = 10_000 * 10 ** 9;

    const mintTx = await mintTo(
      //@ts-expect-error - Type error in spl-token-bankrun dependency
      banksClient,
      employer,
      mint,
      treasuryTokenAccount,
      employer,
      amount
    );
    console.log('Mint Treasury Token Account:', mintTx);
  });
  it("should create an employee vesting account", async () => {
   
    console.log("Company name length:", company.length); 
    if (company.length > 32) {
      throw new Error("Company name exceeds maximum allowed length of 32 characters.");
    }
    console.log("Company name length:", company.length); 
    const tx2 = await program.methods
    .createEmployeeAccount(
      new BN(0),
      new BN(100),
      new BN(100),
      new BN(0),
      company)
    .accounts({
      beneficiary: beneficiary.publicKey,
      tokenvestingAccount: tokenvestingAccountKey,
       })
    .rpc({commitment: 'confirmed', skipPreflight: true});

    console.log('Create employee account:', tx2);
    console.log('Employee account key:', beneficiary.publicKey.toBase58());

  });
  it("should claim the employee's vested tokens", async () => {
    await new Promise((resolve) => setTimeout(resolve, 1000));
        // Get current clock (BanksClient method)
    const currentClock = await context.banksClient.getClock();

    // Set new time (+1 day example)
    const newClock = new Clock(
      currentClock.slot,
      currentClock.epochStartTimestamp,
      currentClock.epoch,
      currentClock.leaderScheduleEpoch,
      currentClock.unixTimestamp + 86_400n // Add 86400 seconds (1 day)
    );

    await context.setClock(newClock);
  
    // 3. Execute claimTokens instruction
    const tx3 = await program2.methods
    .claimTokens("company")
    .accounts({
      tokenProgram: TOKEN_PROGRAM_ID,
     }  
    )
    .rpc({commitment: 'confirmed'});
  
     console.log("Claim tokens transaction:", tx3);
  
    
    });

});


