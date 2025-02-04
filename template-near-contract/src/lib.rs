use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::serde::{ Deserialize, Serialize };
use near_sdk::collections::{ LookupMap, UnorderedMap };
use near_sdk::{ env, near_bindgen, AccountId, Balance, PanicOnDefault, Promise };


pub type FreeLancerId = AccountId;
pub type ClientId = AccountId;
pub type JobId = String;

#[derive(Deserialize, BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum Status {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
}

pub trait OutSourcing {
    // Đăng ký làm freelancer.
    fn register_executor(&mut self, fullname: String, skills: Vec<String>) -> FreeLancer;
    // Đăng ký để làm người giao job.
    fn register_client(&mut self, organization_name: String, industry: String) -> Client;
    // Client -> Tạo Jobs
    fn create_job(&mut self, title: String, desc: String, budget: Balance, tags: Vec<String>, duration: u64) -> Job;
    // Freelancer -> Take.
    fn take_job(&mut self, job_id: JobId) -> Job;
    // Update
    fn update_job(&mut self, job_id: JobId, title: Option<String>, desc: Option<String>, budget: Option<Balance>, tags: Option<Vec<String>>, duration: Option<u64>) -> Job;
    // Payment
    
    fn payment(&mut self, price: Balance) -> Promise;
    // View
    fn view_all_jobs(&self) -> Vec<Job>;
    fn view_job_by_id(&self, job_id: JobId) -> Job;

    fn view_freelancer_by_id(&self) -> FreeLancer;
}


#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Job {
    pub job_id: JobId,
    pub author: ClientId,
    pub executor: Option<FreeLancerId>,
    pub title: String,
    pub desc: String,
    pub budget: Balance,
    pub tags: Vec<String>,
    pub created_at: String,
    pub duration: u64,
    pub status: Status,
}


#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FreeLancer {
    pub freelancer_id: FreeLancerId,
    pub fullname: String,
    pub skills: Vec<String>,
    pub availability: Option<bool>,
    pub credit: Balance,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Client {
    pub client_id: ClientId,
    pub organization_name: String,
    pub industry: String,
    pub credit: Balance,
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner: AccountId,

    // Jobs
    pub all_jobs: UnorderedMap<u128, Job>,
    pub job_by_id: LookupMap<JobId, Job>,
    pub jobs_by_executor: LookupMap<FreeLancerId, Job>,
    pub jobs_by_owner: LookupMap<ClientId, Job>,
    pub total_jobs: u128,

    // Freelancers
    pub all_freelancers: UnorderedMap<u128, FreeLancer>,
    pub freelancer_by_id: LookupMap<FreeLancerId, FreeLancer>,
    pub total_freelancers: u128,

    // Clients
    pub all_clients: UnorderedMap<u128, Client>,
    pub client_by_id: LookupMap<ClientId, Client>,
    pub total_clients: u128,
}




// Nhớ là phân insert,



// Implement the contract structure
#[near_bindgen]
impl OutSourcing for Contract {
    
    fn register_executor(&mut self, fullname: String, skills: Vec<String>) -> FreeLancer {
        let id = env::signer_account_id();
        assert!(!self.freelancer_by_id.contains_key(&id), "This Freelancer has already existed");

        let new_freelancer = FreeLancer {
			freelancer_id: env::signer_account_id(),
            fullname,
            skills,
            credit: 0,
            availability: Some(true),
        };
		
        
		self.total_freelancers += 1;
		self.freelancer_by_id.insert(&id, &new_freelancer);
        self.all_freelancers.insert(&self.total_freelancers, &new_freelancer);
        new_freelancer
    }

    fn register_client(&mut self, organization_name: String, industry: String) -> Client {
        let id = env::signer_account_id();
        assert!(!self.client_by_id.contains_key(&id), "This Client has been already existed");

        let new_client = Client {
			client_id: env::signer_account_id(),
            organization_name,
            industry,
            credit: 0,
        };
		
        
		self.total_clients += 1;
		self.client_by_id.insert(&id, &new_client);
        self.all_clients.insert(&self.total_clients, &new_client);
        new_client
    }
    fn create_job(&mut self, title: String, desc: String, budget: Balance, tags: Vec<String>, duration: u64) -> Job {
        let id = env::signer_account_id();
        assert!(self.client_by_id.contains_key(&id), "Register first to be a Client");

        let _current = env::block_timestamp().to_string();
    
        let new_job = Job {
            job_id: "J-".to_string() + _current.as_str(),
            author: env::signer_account_id(),
            executor: None,
            title, desc, budget, tags, duration,
            created_at: _current,
            status: Status::Open
        };

        self.total_jobs += 1;
        self.all_jobs.insert(&self.total_jobs, &new_job);
        self.job_by_id.insert(&new_job.job_id, &new_job);
        self.jobs_by_owner.insert(&env::signer_account_id(), &new_job);
        new_job
    }
    fn take_job(&mut self, job_id: JobId) -> Job {
        assert!(self.job_by_id.contains_key(&job_id), "This job doesn't exist");

        let id = env::signer_account_id();

        assert!(self.freelancer_by_id.contains_key(&id), "Register first to be a Freelancer");

        // let mut job = self.job_by_id.get(&job_id).unwrap();
        let mut job = self.view_job_by_id(job_id.clone());
        job.executor = Some(id.clone());
        job.status = Status::InProgress;

        self.job_by_id.insert(&job_id, &job);
        self.jobs_by_executor.insert(&id, &job);
        self.jobs_by_owner.insert(&job.author,  &job);
        job
    }
    fn update_job(&mut self, job_id: JobId, title: Option<String>, desc: Option<String>, budget: Option<Balance>, tags: Option<Vec<String>>, duration: Option<u64>) -> Job {
        assert!(self.job_by_id.contains_key(&job_id), "This job doesn't exist");

        let mut job = self.view_job_by_id(job_id.clone());
        
        if let Some(new_title) = title {
            job.title = new_title;
        }
        if let Some(new_desc) = desc {
            job.desc = new_desc;
        }
        if let Some(new_budget) = budget {
            job.budget = new_budget;
        }
        if let Some(new_tags) = tags {
            job.tags = new_tags;
        }
        if let Some(new_duration) = duration {
            job.duration = new_duration;
        }
        
        // Update the job in the data structures
        self.job_by_id.insert(&job_id, &job);
        self.jobs_by_owner.insert(&job.author, &job);
        job
    }
    fn view_all_jobs(&self) -> Vec<Job> {
        let mut jobs = Vec::new();
        
        for i in 1..self.all_jobs.len() + 1 {
            if let Some(job) = self.all_jobs.get(&(i as u128)) {
                jobs.push(job);
            }
        }

        jobs
    }
    fn view_job_by_id(&self, job_id: JobId) -> Job {
        self.job_by_id.get(&job_id).unwrap()
    }
    #[payable]
    fn payment(&mut self, price: Balance) -> Promise {
        // Assert!
        // Price == env::attached_deposit();
        assert_eq!(self.owner, env::signer_account_id(), "You are not owner");
        let conversion_ratio: Balance = 1000000000000000000000000;
        assert_eq!(price, env::attached_deposit() / conversion_ratio, "Not enough currecy");

        Promise::new(env::signer_account_id()).transfer(price)
    }

    fn view_freelancer_by_id(&self) -> FreeLancer {
        let id = env::signer_account_id();
        self.freelancer_by_id.get(&id).unwrap()
    }
}


#[near_bindgen]
impl Contract {
    #[init]
    pub fn init() -> Self {
        Self {
            owner: env::signer_account_id(),

            all_jobs: UnorderedMap::new(b"all jobs".try_to_vec().unwrap()),
            job_by_id: LookupMap::new(b"job by id".try_to_vec().unwrap()),
            jobs_by_executor: LookupMap::new(b"jobs by executor".try_to_vec().unwrap()),
            jobs_by_owner: LookupMap::new(b"jobs by owner".try_to_vec().unwrap()),
            total_jobs: 0,

            all_freelancers: UnorderedMap::new(b"all freelancers".try_to_vec().unwrap()),
            freelancer_by_id: LookupMap::new(b"freelancer by id".try_to_vec().unwrap()),
            total_freelancers: 0,

            all_clients: UnorderedMap::new(b"all clients".try_to_vec().unwrap()),
            client_by_id: LookupMap::new(b"client by id".try_to_vec().unwrap()),
            total_clients: 0,
        }
    }
}