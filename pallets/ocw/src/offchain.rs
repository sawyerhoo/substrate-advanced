use codec::{Decode, Encode};
use serde::Deserialize;
use sp_core::offchain::Duration;
use sp_runtime::offchain::http;
use sp_std::vec::Vec;
use sp_io::offchain_index;

#[derive(Deserialize, Encode, Decode, Debug)]
pub struct RepoInfo {
	pub stargazers_count: u64,
}

#[derive(Debug, Default, Encode, Decode)]
pub struct IndexingData(Vec<u8>, u64);


#[derive(Debug, Default, Encode, Decode, Deserialize)]
pub struct PriceStructData {
	pub price: u32,
}


pub fn get_count() -> Result<u32,http::Error>{
	let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
	
	let request =
	sp_runtime::offchain::http::Request::get("http://mock.apistub.cn/user/Sawyer/SampleProject/SampleAPI");
	
	let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;

	let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
	// Let's check the status code before we proceed to reading the response.
	if response.code != 200 {
		log::warn!("### Unexpected status code: {}", response.code);
		return Err(http::Error::Unknown)
	}

	// Next we want to fully read the response body and collect it to a vector of bytes.
	// Note that the return object allows you to read the body in chunks as well
	// with a way to control the deadline.
	let body = response.body().collect::<Vec<u8>>();

	// Create a str slice from the body.
	let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
		log::warn!("No UTF8 body");
		http::Error::Unknown
	})?;

	log::warn!("### Got response: body_str= {:?}", body_str);

	let price_struct: PriceStructData = serde_json::from_str(body_str).map_err(|_| {
		log::warn!("No UTF8 body");
		http::Error::Unknown
	})?;
	log::warn!("### Got response: price_struct= {:?}", price_struct);


	let price = price_struct.price;

	log::warn!("Got price: {} cents", price);

	Ok(price)
}

pub fn fetch_repo_info() -> Result<RepoInfo, http::Error> {
	// prepare for send request
	let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(8_000));
	let request = http::Request::get("https://api.github.com/repos/paritytech/substrate");
	let pending = request
		.add_header("User-Agent", "Substrate-Offchain-Worker")
		.deadline(deadline)
		.send()
		.map_err(|_| http::Error::IoError)?;
	let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
	if response.code != 200 {
		log::warn!("Unexpected status code: {}", response.code);
		return Err(http::Error::Unknown)
	}
	let body = response.body().collect::<Vec<u8>>();
	let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
		log::warn!("No UTF8 body");
		http::Error::Unknown
	})?;

	// parse the response str
	let repo_info: RepoInfo = serde_json::from_str(body_str).map_err(|_| http::Error::Unknown)?;

	Ok(repo_info)
}

pub fn derived_key<BN>(block_number: BN, key: &[u8]) -> Vec<u8>
where
	BN: Encode,
{
	block_number.using_encoded(|encoded_bn| {
		key.iter().chain(b"@".iter()).chain(encoded_bn).copied().collect::<Vec<u8>>()
	})
}



pub fn offchain_index_set<BN>(block_number: BN, number: u64)
    where BN: Encode,
    {
	let key = derived_key(block_number, b"indexing_1");
	let data = IndexingData(b"submit_number_unsigned".to_vec(), number).encode();
	log::info!("offchain_index::set, key: {:?}, data: {:?}", key, data);
	offchain_index::set(&key, &data);
}