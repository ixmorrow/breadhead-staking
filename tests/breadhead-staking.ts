import * as anchor from "@project-serum/anchor"
import { Program } from "@project-serum/anchor"
import { PublicKey, SystemProgram, Keypair } from '@solana/web3.js'
import { BreadheadStaking } from "../target/types/breadhead_staking"
import { IDENTIFIER_SEED, STAKE_POOL_SEED, STAKE_ENTRY_SEED } from '../src/stakePool/const'
import { createNFTMint, createMasterEditionTxs } from '../src/stakePool/utils'
import { getAssociatedTokenAddress, getAccount } from '@solana/spl-token'
import { TOKEN_PROGRAM_ID } from "@project-serum/anchor/dist/cjs/utils/token"
import { PrimarySaleCanOnlyBeFlippedToTrueError, PROGRAM_ID as METADATA_PROGRAM_ID } from '@metaplex-foundation/mpl-token-metadata'
import { BN } from "bn.js"
import { assert } from "chai"
import { token } from "@project-serum/anchor/dist/cjs/utils"

describe("breadhead-staking", async() => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env())

  const program = anchor.workspace.BreadheadStaking as Program<BreadheadStaking>
  const provider = anchor.AnchorProvider.env()

  const [identifier, identifierBump] = await PublicKey.findProgramAddress([Buffer.from(IDENTIFIER_SEED)], program.programId)
  let stakePool: PublicKey = null
  let originalMint: PublicKey = null
  let metadataInfo: [PublicKey, PublicKey] = null

  const nftAuthority = Keypair.generate()

  it("Create nft", async () => {
     // create original mint
      originalMint = await createNFTMint(
        provider.connection,
        nftAuthority,
        provider.wallet.publicKey,
      )

      // metadata and master edition
      metadataInfo = await createMasterEditionTxs(
        originalMint,
        nftAuthority,
        provider.connection
      )
  })

  it("Initialize Identifier", async () => {
    let identifierAcct = await program.account.identifier.fetch(identifier)

    if (identifierAcct == null) {
      const tx = await program.methods.initIdentifier()
      .accounts({
        identifier: identifier,
        payer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId
      })
      .rpc()
      console.log("Init Identifier tx: ", tx)
    }
  })

  it("Initialize stake pool", async () => {
    let identifierAcct = await program.account.identifier.fetch(identifier)
    const [stakePoolId, stakePoolBump] = await PublicKey.findProgramAddress(
      [Buffer.from(STAKE_POOL_SEED), identifierAcct.count.toArrayLike(Buffer, "le", 8)],
      program.programId
    )
    stakePool = stakePoolId

    const tx = await program.methods.initPool({
      overlayText: "",
      imageUri: "",
      requiresCollections: [],
      requiresCreators: [],
      requiresAuthorization: false,
      authority: provider.wallet.publicKey,
      resetOnStake: false,
      cooldownSeconds: null,
      minStakeSeconds: null,
      endDate: null,
    })
    .accounts({
      stakePool: stakePool,
      identifier: identifier,
      payer: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId
    })
    .rpc()
  })

  it('Create stake entry', async () => {
    const [stakeEntry, entryBump] = await PublicKey.findProgramAddress(
      [Buffer.from(STAKE_ENTRY_SEED), stakePool.toBytes(), originalMint.toBuffer(), PublicKey.default.toBuffer()],
      program.programId
    )

    const tx = await program.methods.initEntry(provider.wallet.publicKey)
    .accounts({
      stakeEntry: stakeEntry,
      stakePool: stakePool,
      originalMint: originalMint,
      originalMintMetadata: metadataInfo[0],
      payer: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId
    })
    .rpc()

  })

  it('Stake NFT', async () => {
    const [stakeEntry, entryBump] = await PublicKey.findProgramAddress(
      [Buffer.from(STAKE_ENTRY_SEED), stakePool.toBytes(), originalMint.toBuffer(), PublicKey.default.toBuffer()],
      program.programId
    )

    const [programAuthority, authBump] = await PublicKey.findProgramAddress(
      [Buffer.from("authority")],
      program.programId
    )

    const [stakeState, stateBump] = await PublicKey.findProgramAddress(
      [provider.wallet.publicKey.toBuffer(), originalMint.toBuffer(), Buffer.from("state")],
      program.programId
    )

    const userAta = await getAssociatedTokenAddress(originalMint, provider.wallet.publicKey)

    const tx = await program.methods.stake(new BN(1))
    .accounts({
      stakeEntry: stakeEntry,
      stakePool: stakePool,
      programAuthority: programAuthority,
      originalMint: originalMint,
      masterEdition: metadataInfo[1],
      user: provider.wallet.publicKey,
      userOriginalMintTokenAccount: userAta,
      stakeState: stakeState,
      tokenProgram: TOKEN_PROGRAM_ID,
      metadataProgram: METADATA_PROGRAM_ID
    })
    .rpc()

    const tokenAccount = await getAccount(provider.connection, userAta)
    assert(tokenAccount.isFrozen, 'token account is not frozen')
    assert(tokenAccount.delegate.toBase58() == programAuthority.toBase58(), 'delegate does not match')
    assert(tokenAccount.owner.toBase58() == provider.wallet.publicKey.toBase58(), 'token account owner does not match')
  })

  it('Unstake nft', async () => {
    const [stakeEntry, entryBump] = await PublicKey.findProgramAddress(
      [Buffer.from(STAKE_ENTRY_SEED), stakePool.toBytes(), originalMint.toBuffer(), PublicKey.default.toBuffer()],
      program.programId
    )

    const [programAuthority, authBump] = await PublicKey.findProgramAddress(
      [Buffer.from("authority")],
      program.programId
    )

    const [stakeState, stateBump] = await PublicKey.findProgramAddress(
      [provider.wallet.publicKey.toBuffer(), originalMint.toBuffer(), Buffer.from("state")],
      program.programId
    )

    const userAta = await getAssociatedTokenAddress(originalMint, provider.wallet.publicKey)

    const tx = await program.methods.unstake()
    .accounts({
      stakeEntry: stakeEntry,
      stakePool: stakePool,
      programAuthority: programAuthority,
      originalMint: originalMint,
      masterEdition: metadataInfo[1],
      user: provider.wallet.publicKey,
      userOriginalMintTokenAccount: userAta,
      stakeState: stakeState,
      tokenProgram: TOKEN_PROGRAM_ID,
      metadataProgram: METADATA_PROGRAM_ID
    })
    .rpc()

    const tokenAccount = await getAccount(provider.connection, userAta)
    assert(!tokenAccount.isFrozen, 'token account is still frozen')
    assert(tokenAccount.delegate == null, 'delegate does not match')
    assert(tokenAccount.owner.toBase58() == provider.wallet.publicKey.toBase58(), 'token account owner does not match')
  })
})
