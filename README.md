<!-- markdownlint-disable MD029 MD033 MD041 -->
<div id="top"></div>
<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->

<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]

<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/JeremieRodon/demo-rust-lambda">
    <img src="images/logo.png" alt="Logo" width="712" height="300">
  </a>

  <h3 align="center">Demo Rust on Lambda</h3>

  <p align="center">
    A demonstration of how minimal an effort it takes to use Rust instead of Python for Serverless projects<br/>such as an API Gateway with Lambda functions.
  </p>
</div>

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#about-the-project">About The Project</a></li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#preparation">Preparation</a></li>
        <li><a href="#deployment">Deployment</a></li>
        <li><a href="#cleanup">Cleanup</a></li>
      </ul>
    </li>
    <li>
      <a href="#usage">Usage</a>
      <ul>
        <li><a href="#generating-traffic-on-the-apis">Generating traffic on the APIs</a></li>
        <li><a href="#exploring-the-results-with-cloudwatch-log-insights">Exploring the results with CloudWatch Log Insights</a></li>
      </ul>
    </li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
  </ol>
</details>

<!-- ABOUT THE PROJECT -->
## About The Project

### Functional requirements

As an example, we are creating a backend-API that models a ***Sheep Shed***.

The ***Sheep Shed*** is housing... well... **Sheeps**. Each ***Sheep*** has a ***Tattoo*** which is unique: it is a functionnal error to have sheeps sharing the same tattoo. A ***Sheep*** also have a ***Weight***, which is important down the road.

The ***Sheep Shed*** obviously has a **Dog** that can, when asked, count the sheeps in the shed.

The ***Sheep Shed*** unfortunately has a hungry **Wolf** lurking around, who wants to eat the sheeps. This wolf is quite strange, he suffers from Obsessive-Compulsive Disorder.
Even starving, he can only eat a ***Sheep*** if its ***Weight*** expressed in micrograms is a prime number. And of course, if multiple sheeps comply with his OCD he wants the heaviest one!

Finally, the ***Sheep Shed*** has a resident **Cat**. The cat does not care about the sheeps, shed, wolf or dog. He is interested only in its own business. This is a savant cat that
recently took interest in a [2-ary variant of the Ackermann function](https://en.wikipedia.org/wiki/Ackermann_function#TRS,_based_on_2-ary_function). The only way to currently get his
attention is to ask him about it.

### AWS design

The ***Sheep Shed*** is accessible through an **Amazon API Gateway** exposing 4 paths:

- POST /sheeps/`<Tattoo>` to add a sheep in the shed with the given `Tattoo` and a random weight generated by the API
- GET /dog to retrieve the current sheep count
- GET /cat?m=`<m>`&n=`<n>` to ask the cat to compute the Ackermann function for given `m` and `n`
- DELETE /wolf to trigger a raid on the shed by our OCD wolf.

Each of these paths has its own **AWS Lambda** function.

The backend is an **Amazon DynamoDB** table.

### But... Why?

Ok that's just a demo but the crux of it is:

- The **Dog** performs a task that only require to scan DynamoDB with virtually no operationnal overhead other than driving the scan pagination
- The **sheep insertion** performs a random number generation, but is also almost entirely tied to a single DynamoDB interaction (PutItem)
- The **Wolf** require to not only scan the entire DynamoDB table, but also to compute the prime numbers to be able to efficiently test if the weight of each sheep is itself a prime number
then, if a suitable sheep is found, he eats (DeleteItem) it
- The **Cat** performs a purely algorithmic task with no I/O required.

As a result, we can compare the size of the advantage of Rust over Python in these various situations.

*NB1: The DynamoDB table layout is intentionaly bad: it would be possible to create indexes to drastically accelerate the search of a suitable sheep for the wolf, but that's not the subject of
this demonstration*

*NB2: Initially I thought that activities tied to DynamoDB (Network I/O) operations would greatly reduce the advantage of Rust over Python (because
packets don't go faster between Lambda and DynamoDB depending on the language used). But it turns out that even for "pure" IO bound activities Rust
lambdas are crushing Python lambdas...*

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- GETTING STARTED -->
## Getting Started

You can easily deploy the demo in you own AWS account in less than 15 minutes. The cost of deploying and loadtesting will
be less than $1: CodePipeline/CodeBuild will stay well within their Free-tier; API Gateway, Lambda and DynamoDB
are all in pay-per-request mode at an aggregated rate of ~$5/million req and you will make a few tens of thousands
of request (it will cost you pennies).

Here is an overview of what will be deployed:
<div align="center">
    <img src="images/architecture.png" alt="Architecture">
    <p><i>This PNG can be edited using <a href="https://draw.io">Draw.io</a></i></p>
</div>

### Prerequisites

You need **an existing AWS account**, with permissions to use the following services:

- AWS CodeStar Connections
- AWS CloudFormation
- AWS CodePipeline
- Amazon Simple Storage Service (S3)
- AWS CodeBuild
- AWS Lambda
- Amazon API Gateway
- Amazon DynamoDB
- AWS IAM (roles will be created for Lambda, CodeBuild, CodePipeline and CloudFormation)
- Amazon CloudWatch Logs

You also need **a GitHub account**, as the deployment method I propose here rely on you being able to fork this repository (CodePipeline only accepts source GitHub repositories that you own for obvious security reasons).

### Preparation

#### 1. Fork the repo

Fork this repository in you own GitHub account. Copy the ID of the new repository (\<UserName>/demo-rust-lambda), you will need it later. Be mindfull of the case.

The simplest technique is to copy it from the browser URL:

![Step 0](images/get-started-0.png)

#### Important

In the following instructions, there is an *implicit* instruction to **always ensure your AWS Console
is set on the AWS Region you intend to use**. You can use any region you like, just stick to it.

#### 2. Create a CodeStar connection to your GitHub account

This step is only necessary if you don't already have a CodeStar Connection to your GitHub account. If you do, you can reuse it: just retrieve its ARN and keep it on hand.

1. Go to the CodePipeline console, select Settings > Connections, use the GitHub provider, choose any name you like, click Connect to GitHub

![Step 1](images/get-started-1.png)

2. Assuming you were already logged-in on GitHub, it will ask you if you consent to let AWS do stuff in your GitHub account. Yes you do.

![Step 2](images/get-started-2.png)

3. You will be brought back to the AWS Console. Choose the GitHub Apps that was created for you in the list (don't mind the number on the screenshot, yours will be different), then click Connect.

![Step 3](images/get-started-3.png)

4. The connection is now created, copy its ARN somewhere, you will need it later.

![Step 4](images/get-started-4.png)

### Deployment

Now you are ready to deploy, download the CloudFormation template [ci-template.yml](https://github.com/JeremieRodon/demo-rust-lambda/blob/master/ci-template.yml)
from the link or from your newly forked repository if you prefer.

5. Go to the CloudFormation console and create a new stack.

![Step 5](images/get-started-5.png)

6. Ensure *Template is ready* is selected and *Upload a template file*, then specify the `ci-template.yml` template that you just downloaded.

![Step 6](images/get-started-6.png)

7. Choose any Stack name you like, set your CodeStar Connection Arn (previously copied) in `CodeStarConnectionArn` and your forked repository ID in `ForkedRepoId`

![Step 7](images/get-started-7.png)

8. Skip the *Configure stack options*, leaving everything unchanged

9. At the *Review and create* stage, acknowledge that CloudFormation will create roles and Submit.

![Step 8](images/get-started-8.png)

At this point, everything will roll on its own, the full deployment should take ~8 minutes, largely due to the quite long first compilation of Rust lambdas.

If you whish to follow what is happening, keep the CloudFormation tab open in your browser and open another one on the CodePipeline console.

### Cleanup

To cleanup the demo resources, you need to remove the CloudFormation stacks **IN ORDER**:

- **First** remove the two API stacks named `<ProjectName>-rust-api` and `<ProjectName>-python-api`
- **/!\\ Wait until both are successfully removed /!\\**
- **Then** remove the CICD stack (the one you created yourself)

You **MUST** follow that order of operation because the CICD stack owns the IAM Role used by the other two to performs their operation;
therefore destroying the CICD stack first will prevent the API stacks from operating.

Removing the CloudFormation stacks correctly will cleanup every resources created for this demo, no further cleanup is needed.

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- USAGE EXAMPLES -->
## Usage

### Generating traffic on the APIs

The `utils` folder of the repository contains scripts to generate traffic on each API. The easiest way is to use `execute_default_benches.sh`:

```sh
cd utils
./execute_default_benches.sh --rust-api <RUST_API_URL> --python-api <PYTHON_API_URL>
```

*You can find the URL of each API (`RUST_API_URL` and `PYTHON_API_URL` in the script above) in the Outputs sections of the respective CloudFormation stacks (stacks of the APIs, not the CICD) or directly in the API Gateway console.*

It will execute a bunch of API calls (~4k/API) and typically takes ~10minutes to run, depending on your internet connection and latence to the APIs.

For reference, here is an execution report with my APIs deployed in the Paris region (as I live there...):

```sh
./execute_default_benches.sh \
--rust-api https://swmafop2bd.execute-api.eu-west-3.amazonaws.com/v1/ \
--python-api https://pv6wlmzjo0.execute-api.eu-west-3.amazonaws.com/v1/
```

It outputs:

```sh
Launching test...
###############################################################
# This will take a while and appear to hang, but don't worry! #
###############################################################
PYTHON CAT: ./invoke_cat.sh https://pv6wlmzjo0.execute-api.eu-west-3.amazonaws.com/v1/
Calls took 72555ms
RUST CAT: ./invoke_cat.sh https://swmafop2bd.execute-api.eu-west-3.amazonaws.com/v1/
Calls took 21036ms
PYTHON SHEEPS: ./insert_sheeps.sh https://pv6wlmzjo0.execute-api.eu-west-3.amazonaws.com/v1/
Insertion took 23430ms
RUST SHEEPS: ./insert_sheeps.sh https://swmafop2bd.execute-api.eu-west-3.amazonaws.com/v1/
Insertion took 23429ms
PYTHON DOG: ./invoke_dog.sh https://pv6wlmzjo0.execute-api.eu-west-3.amazonaws.com/v1/
Calls took 26990ms
RUST DOG: ./invoke_dog.sh https://swmafop2bd.execute-api.eu-west-3.amazonaws.com/v1/
Calls took 23385ms
PYTHON WOLF: ./invoke_wolf.sh https://pv6wlmzjo0.execute-api.eu-west-3.amazonaws.com/v1/
Calls took 194190ms
RUST WOLF: ./invoke_wolf.sh https://swmafop2bd.execute-api.eu-west-3.amazonaws.com/v1/
Calls took 27279ms
Done.
```

Of course, you can also play with the individual scripts of the `utils` folder, just invoke them with `--help` to see what you can do with them:

```sh
./invoke_cat.sh --help
```

```sh
Usage: ./invoke_cat.sh [<OPTIONS>] <API_URL>

Repeatedly call GET <API_URL>/cat?m=<m>&n=<n> with m=3 and n=8 unless overritten

-p|--parallel <task_count>      The number of concurrent task to use. (Default: 100)
-c|--call-count <count> The number of call to make. (Default: 1000)
-m <integer>    The 'm' number for the Ackermann algorithm. (Default: 3)
-n <integer>    The 'n' number for the Ackermann algorithm. (Default: 8)

OPTIONS:
-h|--help                       Show this help
```

### Exploring the results with CloudWatch Log Insights

After you generated load, you can compare the performance of the lambdas using CloudWatch Log Insights.

Go to the CloudWatch Log Insights console, set the date/time range appropriately, select the 8 log groups of our Lambdas (4 Rust, 4 Python) and set the query:

![Log-Insights](images/log-insights.png)

Here is the query:

```text
filter @type = "REPORT"
| fields greatest(@initDuration, 0) + @duration as duration, ispresent(@initDuration) as coldStart
| parse @log /^\d+:.*?-(?<Lambda>(rust|python)-.+)$/
| stats count(*) as count, 
avg(duration) as avgDuration, min(duration) as minDuration, max(duration) as maxDuration, stddev(duration) as StdDevDuration,
avg(@billedDuration) as avgBilled, min(@billedDuration) as minBilled, max(@billedDuration) as maxBilled, stddev(@billedDuration) as StdDevBilled,
avg(@maxMemoryUsed / 1024 / 1024) as avgRam, min(@maxMemoryUsed / 1024 / 1024) as minRam, max(@maxMemoryUsed / 1024 / 1024) as maxRam, stddev(@maxMemoryUsed / 1024 / 1024) as StdDevRam
by Lambda, coldStart
```

This query gives you the average, min, max and standard deviation for 3 metrics: duration, billed duration and memory used. Result are grouped by lambda function and separated between coldstart and non-coldstart runs.

And here are the results yielded by my tests (Duration: ms, Billed: ms, Ram: MB; StdDev removed for bievety):

---

| Lambda |-| coldStart | count |-| avgDuration | minDuration | maxDuration |-| avgBilled | minBilled | maxBilled |-| avgRam | minRam | maxRam |
| --- |-| --- | --- |-| --- | --- | --- |-| --- | --- | --- |-| --- | --- | --- |
| rust-delete-wolf-ocd |-| no | 1256 |-| 95.1301 | 42.8 | 389.74 |-| 95.6274 | 43 | 390 |-| 25.2633 | 22.8882 | 26.7029 |
| rust-delete-wolf-ocd |-| yes | 54 |-| 350.7987 | 328.48 | 371.89 |-| 351.2778 | 329 | 372 |-| 21.9698 | 21.9345 | 22.8882 |
| python-delete-wolf-ocd |-| no | 2289 |-| 4271.3851 | 2243.41 | 7006.29 |-| 4271.8873 | 2244 | 7007 |-| 88.5117 | 83.9233 | 92.5064 |
| python-delete-wolf-ocd |-| yes | 102 |-| 9261.8528 | 7984.55 | 9601.49 |-| 8976.1667 | 7700 | 9298 |-| 80.8192 | 80.1086 | 81.0623 |
| rust-get-dog-count |-| no | 988 |-| 17.4312 | 13.05 | 39.5 |-| 17.9362 | 14 | 40 |-| 23.6073 | 21.9345 | 23.8419 |
| rust-get-dog-count |-| yes | 12 |-| 193.2633 | 182.49 | 218.09 |-| 193.9167 | 183 | 219 |-| 21.5371 | 20.9808 | 21.9345 |
| python-get-dog-count |-| no | 900 |-| 669.4292 | 603.35 | 845.58 |-| 669.9178 | 604 | 846 |-| 76.8926 | 76.2939 | 78.2013 |
| python-get-dog-count |-| yes | 100 |-| 3003.9024 | 2796.17 | 3204.47 |-| 2713.36 | 2522 | 2914 |-| 76.2939 | 76.2939 | 76.2939 |
| rust-post-sheep-random |-| no | 989 |-| 8.1096 | 4.67 | 76.32 |-| 8.5875 | 5 | 77 |-| 22.8351 | 20.9808 | 23.8419 |
| rust-post-sheep-random |-| yes | 11 |-| 149.8936 | 140.53 | 163.69 |-| 150.3636 | 141 | 164 |-| 21.0675 | 20.9808 | 21.9345 |
| python-post-sheep-random |-| no | 900 |-| 599.0018 | 549.25 | 693.93 |-| 599.4967 | 550 | 694 |-| 76.7962 | 76.2939 | 78.2013 |
| python-post-sheep-random |-| yes | 100 |-| 2958.1016 | 2823.31 | 3338.99 |-| 2657.59 | 2544 | 2839 |-| 76.2749 | 75.3403 | 77.2476 |
| rust-get-cat-ackermann |-| no | 960 |-| 124.1233 | 90.98 | 174.78 |-| 124.6073 | 91 | 175 |-| 15.8966 | 14.3051 | 16.2125 |
| rust-get-cat-ackermann |-| yes | 40 |-| 150.1008 | 130.29 | 164.68 |-| 150.55 | 131 | 165 |-| 14.3051 | 14.3051 | 14.3051 |
| python-get-cat-ackermann |-| no | 898 |-| 5935.7388 | 5883.13 | 6093.28 |-| 5936.2327 | 5884 | 6094 |-| 29.9335 | 29.5639 | 30.5176 |
| python-get-cat-ackermann |-| yes | 102 |-| 6034.4553 | 5985.93 | 6158.94 |-| 5948.3824 | 5906 | 6071 |-| 29.9379 | 29.5639 | 30.5176 |

---

Kind of speaks for itself, right? Rust is on average **50x faster**, **33x cheaper** and **4x** more memory efficient!

***NB**: Note that Rust is 50x faster but **only** 33x cheaper because when using Rust (or any custom runtime) on Lambda the Cold Start is billed by AWS, whereas with Python (and other native Lambda runtimes) the Cold Start is generaly not billed (for now)*

<p align="right">(<a href="#top">back to top</a>)</p>

### Visualizing the results with GSheets

You can duplicate and use the GSheet document I used to produce the charts of my talk on the subject:

<https://docs.google.com/spreadsheets/d/1F8JGnyyVbkoee2vCYRoXRZlaA4O35NHFVzRBztYR66c/edit?usp=sharing>

If you export the results given by CloudWatch Log Insights as CSV, you can directly paste them in the first Sheet (named `All`). Just **make sure that the `Lambda` and `coldStart` columns are exactly in the same order** as what I had, because the other sheets for the Sheeps, Dogs, etc... are hard-linked to the cells of the first one (there is no "search" going on).

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- LICENSE -->
## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- CONTACT -->
## Contact

Jérémie RODON - <jeremie.rodon@gmail.com>

[![X][twitter-x-shield]][twitter-x-url]

[![LinkedIn][linkedin-shield]][linkedin-url]

Project Link: [https://github.com/JeremieRodon/demo-rust-lambda](https://github.com/JeremieRodon/demo-rust-lambda)

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/JeremieRodon/demo-rust-lambda.svg?style=for-the-badge
[contributors-url]: https://github.com/JeremieRodon/demo-rust-lambda/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/JeremieRodon/demo-rust-lambda.svg?style=for-the-badge
[forks-url]: https://github.com/JeremieRodon/demo-rust-lambda/network/members
[stars-shield]: https://img.shields.io/github/stars/JeremieRodon/demo-rust-lambda.svg?style=for-the-badge
[stars-url]: https://github.com/JeremieRodon/demo-rust-lambda/stargazers
[issues-shield]: https://img.shields.io/github/issues/JeremieRodon/demo-rust-lambda.svg?style=for-the-badge
[issues-url]: https://github.com/JeremieRodon/demo-rust-lambda/issues
[license-shield]: https://img.shields.io/github/license/JeremieRodon/demo-rust-lambda.svg?style=for-the-badge
[license-url]: https://github.com/JeremieRodon/demo-rust-lambda/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/linkedin-0077B5?style=for-the-badge&logo=linkedin&logoColor=white
[linkedin-url]: https://linkedin.com/in/JeremieRodon
[twitter-x-shield]: https://img.shields.io/badge/Twitter/X-000000?style=for-the-badge&logo=x&logoColor=white
[twitter-x-url]: https://twitter.com/JeremieRodon
