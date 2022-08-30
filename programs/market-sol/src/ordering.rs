use {
    anchor_lang::{
        prelude::*,
    },
    anchor_spl::{
        associated_token,
        token,
    },
};
use crate::state::ListOrder;
use crate::state::OrderDetail;

use crate::error::ErrorCode;

pub fn setup_platform(
    ctx: Context<MarketPlatform>,
) -> Result<()> {
    msg!("Call setup platform");
    let list_order = &mut &mut ctx.accounts.list_order_account;
    list_order.list_owner = ctx.accounts.user_owner.key();

    Ok(())
}

pub fn clear_data(
    ctx: Context<ClearData>,
) -> Result<()> {
    msg!("Call clear data");
    let list_order = &mut &mut ctx.accounts.list_order_account;
    if list_order.list_owner.key().to_string().is_empty() {
        list_order.data.clear();
        return Ok(())
    }
    if list_order.list_owner.key() != ctx.accounts.user_owner.key() {
        msg!("Caller not owner data");
        return Err(error!(ErrorCode::NotOwner));
    }
    list_order.data.clear();
    Ok(())
}

pub fn update_order_detail(
    ctx: Context<UpdateOrderDetail>,
    new_price: u64,
) -> Result<()> {
    msg!("Call update order detail");
    let order_account = &mut ctx.accounts.order_account;
    if order_account.seller != ctx.accounts.wallet.key() {
        return Err(error!(ErrorCode::NotOwner));
    }
    order_account.price = new_price;
    msg!("Call update order detail successfully");
    
    Ok(())
}

pub fn create_order(
    ctx: Context<CreateOrder>,
    sale_lamports: u64,
) -> Result<()> {
    msg!("Call create order");

    // Validate logic business
    msg!("Check token must not listing");
    let nft_id = ctx.accounts.mint.key();
    let list_order_data = &mut &mut ctx.accounts.list_order_account.data;
    // This take O(n). Need use better data struture to reduce time
    if list_order_data.contains(&nft_id) {
        return Err(error!(ErrorCode::AlreadyListing));
    }

    msg!("Creating vault token account...");
    msg!("Vault Token Address: {}", &ctx.accounts.vault_token_account.key());
    associated_token::create(
        CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            associated_token::Create {
                payer: ctx.accounts.owner_authority.to_account_info(),
                associated_token: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        ),
    )?;

    msg!("Transferring NFT...");
    msg!("Owner Token Address: {}", &ctx.accounts.owner_token_account.key());    
    msg!("Vault Token Address: {}", &ctx.accounts.vault_token_account.key());    
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.owner_token_account.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.owner_authority.to_account_info(),
            }
        ),
        1
    )?;
    msg!("NFT transferred successfully");


    msg!("Creating order detail data on chain");
    let order_account = &mut ctx.accounts.order_account;
    order_account.seller = ctx.accounts.owner_authority.key();
    order_account.token_id = nft_id;
    order_account.price = sale_lamports;

    msg!("Push an order");
    list_order_data.push(nft_id);
    
    msg!("Listing completed successfully");

    Ok(())
}


#[derive(Accounts)]
pub struct MarketPlatform<'info> {
    #[account(init, payer = user_owner, space = 9000)]
    pub list_order_account: Account<'info, ListOrder>,
    #[account(mut)]
    pub user_owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClearData<'info> {
    #[account(mut)]
    pub list_order_account: Account<'info, ListOrder>,
    #[account(mut)]
    pub user_owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateOrderDetail<'info> {
    #[account(mut)]
    pub order_account: Account<'info, OrderDetail>,
    #[account(mut)]
    pub wallet: Signer<'info>,
}

#[derive(Accounts)]
pub struct CreateOrder<'info> {
    #[account(
        init,
        payer = owner_authority,
        space = 82,
        seeds = [
            mint.key().as_ref(),
            b"_",
        ],
        bump
    )]
    pub order_account: Account<'info, OrderDetail>,
    #[account(mut)]
    pub list_order_account: Account<'info, ListOrder>,
    #[account(mut)]
    pub mint: Account<'info, token::Mint>,
    #[account(mut)]
    pub owner_token_account: Account<'info, token::TokenAccount>,
    #[account(mut)]
    pub owner_authority: Signer<'info>,
    /// CHECK: We're about to create this with Anchor
    #[account(mut)]
    pub vault_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub vault_authority: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}

// #[account]
// pub struct OrderDetail {
//     pub seller: Pubkey,
//     pub token_id: Pubkey,
//     pub price: u64,
// }

// #[account]
// pub struct ListOrder {
//     pub list_owner: Pubkey,
//     pub data: Vec<Pubkey>,
// }