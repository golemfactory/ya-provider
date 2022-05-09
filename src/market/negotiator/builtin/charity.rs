use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde_json::Number;

use ya_agreement_utils::{Error, OfferDefinition};
use ya_client::model::NodeId;
use crate::config::globals::GLOBAL_STATE;

use crate::display::EnableDisplay;
use crate::market::negotiator::factory::{AgreementExpirationNegotiatorConfig, CharityConfig};
use crate::market::negotiator::{
    AgreementResult, NegotiationResult, NegotiatorComponent, ProposalView,
};

pub struct CharityComponent {

}

impl CharityComponent {
    pub fn new() -> anyhow::Result<Self> {

        Ok(Self {})
    }
}

static CHARITY_WALLET_PROPERTY: &'static str = "golem.com.payment.charity.charity-wallet";

static CHARITY_PERCENTAGE_PROPERTY: &'static str = "golem.com.payment.charity.charity-percentage";

static CHARITY_CONFIRMATION_PROPERTY: &'static str = "golem.com.payment.charity.charity-confirmation";

fn extract_charity_confirmation(proposal: &ProposalView) -> Result<Option<bool>> {
    match proposal.pointer_typed::<bool>(CHARITY_CONFIRMATION_PROPERTY) {
        // Requestor is able to accept Charities, because he set this property.
        Ok(deadline) => Ok(Some(deadline)),
        // If he didn't set this property, he is unable to accept DebitNotes.
        Err(Error::NoKey { .. }) => Ok(None),
        // Property has invalid type. We shouldn't continue negotiations, since
        // Requestor probably doesn't understand specification.
        Err(e) => Err(e.into()),
    }
}

impl NegotiatorComponent for CharityComponent {
    fn negotiate_step(
        &mut self,
        demand: &ProposalView,
        mut offer: ProposalView,
    ) -> anyhow::Result<NegotiationResult> {

        let state_read = GLOBAL_STATE.read().unwrap().clone();
        if state_read.charity_percentage.unwrap() == 0.0 {
            return Ok(NegotiationResult::Ready { offer });
        }

        let charity_confirmation = extract_charity_confirmation(demand)?;

        let confirmation = match charity_confirmation {
            Some(confirmation) => confirmation,
            None => false,
        };

        //does not understand charity
        //TODO maybe add an option to ignore lack of charity confirmation? (i know you dont understand charity but i may use you anyway)
        if confirmation == false {
            return Ok(NegotiationResult::Reject {
                message: format!(
                    "Charity protocol not recognized",
                ),
                is_final: true, // when it's too soon we could try later
            });
        }

        Ok(NegotiationResult::Ready { offer })
    }

    fn fill_template(&mut self, mut template: OfferDefinition) -> anyhow::Result<OfferDefinition> {
        let state_read = GLOBAL_STATE.read().unwrap().clone();

        if state_read.charity_percentage.unwrap() == 0.0 {
            return Ok(template);
        }

        template.offer.set_property(
            CHARITY_WALLET_PROPERTY,
            state_read.charity_wallet.unwrap().to_string().into(),
        );

        template.offer.set_property(
            CHARITY_PERCENTAGE_PROPERTY,
            state_read.charity_percentage.unwrap().into(),
        );

        Ok(template)
    }

    fn on_agreement_terminated(
        &mut self,
        _agreement_id: &str,
        _result: &AgreementResult,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_agreement_approved(&mut self, _agreement_id: &str) -> anyhow::Result<()> {
        Ok(())
    }
}
