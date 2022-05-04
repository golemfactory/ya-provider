use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde_json::Number;

use ya_agreement_utils::{Error, OfferDefinition};
use ya_client::model::NodeId;

use crate::display::EnableDisplay;
use crate::market::negotiator::factory::{AgreementExpirationNegotiatorConfig, CharityConfig};
use crate::market::negotiator::{
    AgreementResult, NegotiationResult, NegotiatorComponent, ProposalView,
};

pub struct CharityComponent {
    pub charity_wallet: NodeId,
    pub charity_percentage: f64,
}

impl CharityComponent {
    pub fn new(config: &CharityConfig) -> anyhow::Result<Self> {
        let charity_wallet = config.charity_wallet;
        let charity_percentage = config.charity_percentage;

        Ok(Self {
            charity_wallet,
            charity_percentage,
        })
    }
}

static CHARITY_WALLET_PROPERTY: &'static str = "golem.com.payment.charity.charity-wallet";

static CHARITY_PERCENTAGE_PROPERTY: &'static str = "golem.com.payment.charity.charity-percentage";

static CHARITY_CONFIRMATION_PROPERTY: &'static str = "golem.com.payment.charity.charity-wallet";

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
        let charity_confirmation = extract_charity_confirmation(demand)?;

        let confirmation = match charity_confirmation {
            Some(confirmation) => confirmation,
            None => false,
        };

        //does not understand charity
        //TODO cleanup this reject
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
        template.offer.set_property(
            CHARITY_WALLET_PROPERTY,
            serde_json::Value::String("asd".to_string()), //TODO extract wallet info from globals
        );

        template.offer.set_property(
            CHARITY_PERCENTAGE_PROPERTY,
            serde_json::Value::Number(Number::from_f64(0.0).unwrap()), //TODO extract wallet info from globals
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
