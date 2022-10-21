import * as web3 from '@solana/web3.js'
import { IDENTIFIER_SEED, STAKE_PROGRAM_ADDRESS, STAKE_POOL_SEED, masterEditionSeed, metadataSeed } from './const'
import { BN } from "@project-serum/anchor"
import { TOKEN_PROGRAM_ID, createMint, createAssociatedTokenAccount, mintTo } from "@solana/spl-token"
import {
    createCreateMasterEditionV3Instruction,
    CreateMetadataAccountInstructionArgs,
    createCreateMetadataAccountInstruction,
    CreateMetadataAccountInstructionAccounts,
    Creator,
    CreateMasterEditionV3InstructionAccounts,
    CreateMasterEditionV3InstructionArgs,
    DataV2,
    PROGRAM_ID as METADATA_PROGRAM_ID
} from "@metaplex-foundation/mpl-token-metadata"

/**
 * Finds the identifier id.
 * @returns
 */
export const findIdentifierId = async (): Promise<[web3.PublicKey, number]> => {
    return web3.PublicKey.findProgramAddress(
        [Buffer.from(IDENTIFIER_SEED)],
        STAKE_PROGRAM_ADDRESS
    );
};

/**
 * Finds the stake pool id.
 * @returns
 */
export const findStakePoolId = async (
    identifier: BN
    ): Promise<[web3.PublicKey, number]> => {
        return web3.PublicKey.findProgramAddress(
        [
            Buffer.from(STAKE_POOL_SEED),
            identifier.toArrayLike(Buffer, "le", 8),
        ],
        STAKE_PROGRAM_ADDRESS
    );
};

/**
 * Pay and create mint and token account
 * @param connection
 * @param creator
 * @returns
 */
export const createNFTMint = async (
    connection: web3.Connection,
    payer: web3.Keypair,
    recipient: web3.PublicKey,
    freezeAuthority: web3.PublicKey = payer.publicKey,
    mintAuthority: web3.PublicKey = payer.publicKey,
    ): Promise<web3.PublicKey> => {
    await safeAirdrop(payer.publicKey, connection)
    
    const mint = await createMint(
        connection,
        payer,
        mintAuthority,
        freezeAuthority,
        0
    )
    const tokenAccount = await createAssociatedTokenAccount(
        connection,
        payer,
        mint,
        recipient
    )

    await mintTo(
        connection,
        payer,
        mint,
        tokenAccount,
        payer.publicKey,
        1
    )

    return mint
}

export const createMasterEditionTxs = async (
    mintId: web3.PublicKey,
    tokenCreatorId: web3.Keypair,
    connection: web3.Connection
    ):  Promise<[web3.PublicKey, web3.PublicKey]> => {
    const tx = new web3.Transaction()

    // create metadata account
    const [metadataId, metadataBump] = await web3.PublicKey.findProgramAddress(
        [Buffer.from(metadataSeed), METADATA_PROGRAM_ID.toBuffer(), mintId.toBuffer()],
        METADATA_PROGRAM_ID
    )


    const metadataAccounts: CreateMetadataAccountInstructionAccounts = {
        metadata: metadataId,
        mint: mintId,
        mintAuthority: tokenCreatorId.publicKey,
        payer: tokenCreatorId.publicKey,
        updateAuthority: tokenCreatorId.publicKey
    }
    const creator: Creator = {
        address: tokenCreatorId.publicKey,
        share: 100,
        verified: true
    }
    const data: DataV2 = {
        name: "test",
        symbol: "TST",
        uri: "http://test/",
        sellerFeeBasisPoints: 10,
        creators: [creator],
        collection: null,
        uses: null,
    }
    const metadataArgs: CreateMetadataAccountInstructionArgs = {
        createMetadataAccountArgs: {
            data: data,
            isMutable: false
        }
    }
    const metadataIx = createCreateMetadataAccountInstruction(
        metadataAccounts,
        metadataArgs
    )
    tx.add(metadataIx)

    // create master edition account
    const [masterEditionId, masterEditionBump] = await web3.PublicKey.findProgramAddress(
        [Buffer.from(metadataSeed), METADATA_PROGRAM_ID.toBuffer(), mintId.toBuffer(), Buffer.from(masterEditionSeed)],
        METADATA_PROGRAM_ID
    )


    const masterEditionAccts:CreateMasterEditionV3InstructionAccounts = {
        edition: masterEditionId,
        metadata: metadataId,
        mint: mintId,
        mintAuthority: tokenCreatorId.publicKey,
        payer: tokenCreatorId.publicKey,
        updateAuthority: tokenCreatorId.publicKey
    }
    const masterEditionArgs: CreateMasterEditionV3InstructionArgs = {
        createMasterEditionArgs: {
            maxSupply: new BN(1)
        }
    }
    
    const masterEditionIx = createCreateMasterEditionV3Instruction(
        masterEditionAccts,
        masterEditionArgs
    )
    tx.add(masterEditionIx)

    const signature = await connection.sendTransaction(tx, [tokenCreatorId])

    return [metadataId, masterEditionId]
}

export function delay(ms: number) {
    return new Promise( resolve => setTimeout(resolve, ms) );
}

export async function safeAirdrop(address: web3.PublicKey, connection: web3.Connection) {
    const acctInfo = await connection.getAccountInfo(address, "confirmed")

    if (acctInfo == null || acctInfo.lamports < web3.LAMPORTS_PER_SOL) {
        let signature = await connection.requestAirdrop(
            address,
            web3.LAMPORTS_PER_SOL
        )
        await connection.confirmTransaction(signature)
    }
}