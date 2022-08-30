import * as anchor from "@project-serum/anchor";
// ** Comment this to use solpg imported IDL **
import {
    createKeypairFromFile,
    derivePda,
    generateKeypair,
    showAllOrder,
    updatePriceAllOrder,
} from './util';
import { MarketSol } from "../target/types/market_sol";


describe("buy-nft", async () => {

    const provider = anchor.AnchorProvider.env()
    const wallet = provider.wallet as anchor.Wallet;
    anchor.setProvider(provider);

    const program = anchor.workspace.MarketSol as anchor.Program<MarketSol>;

    var list_order_account: anchor.web3.Keypair;
    var pda;
    const mint: anchor.web3.PublicKey = new anchor.web3.PublicKey(
        "EQQ6gbeUctyb5WJndyUTeZDQpD53rS3FxhsndN9qUqpP"
    );

    // var deployer: anchor.web3.Keypair;
    var vault: anchor.web3.Keypair;
    var buyer: anchor.web3.Keypair;

    before(async function () {
        // list_order_account = await createKeypairFromFile(__dirname + "/keypairs/vault.local.json");
        // order_account = await generateKeypair(provider);

        // console.log(list_order_account.publicKey.toString());
        // console.log(order_account.publicKey.toString());

        list_order_account = await createKeypairFromFile(__dirname + "/keypairs/vault.json");
        // deployer = await createKeypairFromFile("/Users/menduong/.config/solana/id.json");
        buyer = await createKeypairFromFile(__dirname + "/keypairs/buyer.json");
        vault = list_order_account;
        console.log(list_order_account.publicKey.toString());
        console.log("=============Done before initialize=============");
    });

    it("buy", async () => {

        // Test buy last order in list all of order
        // showAllOrder()
        let listOrder = await (await program.account.listOrder.fetch(list_order_account.publicKey)).data;
        console.log("Total: ", listOrder.length);
        let index = listOrder.length - 1;
        // get last order detail
        let nft_id = listOrder[index];
        let pda = await derivePda(nft_id, program);
        let data = await program.account.orderDetail.fetch(pda);
        // console.log(pda);
        console.log(`Last order:    Token_id: ${data.tokenId}   price: ${data.price}  seller: ${data.seller}`); 
        console.log("Processing buying last order...");
        // Testing constants
        const mint: anchor.web3.PublicKey = new anchor.web3.PublicKey(
            data.tokenId
        );

        const seller: anchor.web3.PublicKey = new anchor.web3.PublicKey(
            data.seller
        )
        // const seller: anchor.web3.PublicKey = new anchor.web3.PublicKey(
        //     data.seller
        // );

        // Derive the associated token account address for owner & buyer

        const ownerTokenAddress = await anchor.utils.token.associatedAddress({
            mint: mint,
            owner: vault.publicKey
        });
        const buyerTokenAddress = await anchor.utils.token.associatedAddress({
            mint: mint,
            owner: buyer.publicKey,
        });
        console.log(`Request to buy NFT: ${mint} for ${data.price} lamports.`);
        console.log(`Owner's Token Address: ${ownerTokenAddress}`);
        console.log(`Buyer's Token Address: ${buyerTokenAddress}`);

        // Transact with the "buy" function in our on-chain program
        const price = data.price;
        await program.methods.buy(
            new anchor.BN(price)
        )
            .accounts({
                seller: seller,
                mint: mint,
                listOrderAccount: list_order_account.publicKey,
                ownerTokenAccount: ownerTokenAddress,
                ownerAuthority: vault.publicKey,
                buyerTokenAccount: buyerTokenAddress,
                buyerAuthority: buyer.publicKey,
            })
            .signers([buyer, vault])
            .rpc();
    });
});