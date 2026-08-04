## Distributor Credit Points and Beneficiary Management System

Client - Siddharth Patel 

Architect / Developer = Keyur Patel

This is a Management System dashboard where my client will manage credit points for his business and calculate final discounts based on calculated points and user hirarchy.

User hirarchy - 
- 
- Top level - only 1 person (this is fixed and in future also can not increase)
- 2nd level - 9 persons (Client can configure this in their settings)
- 3rd level - 6 persons (Client can configure this in their settings) 
- 4th level - 3 persons (Client can configure this in their settings)
- So on.. (depth is configurable in settings)

all persons onboarded will have unique id (6 digits) assigned to them during onboarding process and member creation flow.

Credit Points System - 
- 
- 100 points -> 2% slab
- 400 points -> 4% slab
- 1200 points -> 6% slab
- 3000 points -> 8% slab
- 5000 points -> 10% slab
- 7000 points -> 12% slab
- 10,000 points -> 14% slab

Points can be configurable in settings. i.e. 
- I can change 2% slab to 200 points in future. 
- I can change 6% slan to 1000 points in future. 
- so on..

## High level requirement for software - 

- Client wants Search option in home page so that he can easily search by person name or id and it should open that person details along with his hirarchy (upto 1 depth only).
- Client wants hirarchy chart where he can visualize persons working under another person.
- In hirarchy, Client wants only name, id and credit points visible.
- in individual person detail - Client wants name, all point details earned by him and also persons under him upto 1 depth only, total purchase volume, his phone numner, address, and number of legs working under him (upto 1 depth)
- add new member - it should ask for basic person details like name, phone number, email id (if any), reference ID (this is mandetory for hirarchy), and so on..
- points add screen - Clients wants screen where he can easily add credit points for persons by searching based on name / id.
- settings screen - I can configure credit points, slabs and all other configurable details in application.
- Monthly manual reset - clients wants ability to reset this points reset monthly for all persons (RESET SHOULD BE MANUAL). before reset client wants pop up asking for back up data for this month in excel format. without backup reset should not happen. 
- Client wants ability to export month wise data in excel format also, by default name, id, phone number, credit points should get exported but Client can configure what other details he wants to export also.
- Client does not want to use any keyword like sale, purchase, cash etc. which indicated business. instead use generic words like rewards, credit points etc. (It should make sense the purpose it is used for)
- Client wants yearly average of each individual person's business volumn and credit points exportable in excel format. yearly cycle is defined from 1st January to 31st December. (This is configurable in settings)
- Client also wants separate report for those individuals whose yearly average is below 100 points (this is configurable in settings) in excel format. 
- All reports exported should include basic person's details with phone number, business volumn and credit points.


## Calculation logic (CRITICAL)

Conditions for calculations - 

- calculation should happen from bottom to top in hirarchy. and it should propogate to top level.
- 1 point value is 500 Rs (this is configurable in settings).
- Points reset to default (0) for all users on every 1st of Month. (Manuall reset only user will be prompted to reset on every 1st of Month).
- Final earned points is separate from individual points.
- In order to earn Royalty (1% from each users crossing final slab), 3+ (this is configurable in settings) number of persons should have maximum credit points (for example - 10,000 - 14% slab).
- Always pay Royalty in credit points, not in cash.

Here is the scenarion based calculations - 

Scenario 1 -

- let's say Person D, under whom Person A, B and C are working. Person A is having credit point as 300, Person B having 50 and Person C having 1000 and Person D is having 500.
- so business volume for Person D becomes = A (300) + B (50) + C (1000) + D (500)
- Business volume of Person D = 1850 credit points
- As per this credit points Person D comes under 6% slab. 
- Person A is in 2% slab, Person B is in 0% slab, Person C is in 4% slab.
- Now total earned points for D is as below, 
    1. with respect to Person A -> 6% (D's slab) - 2% (A's slab) = 4% of 300 points (A's credit points)
    2. with respect to Person B -> 6% (D's slab) - 0% (B's slab) = 6% of 50 points (B's credit points)
    3. with respect to Person C -> 6% (D's slab) - 4% (C's slab) = 2% of 1000 points (C's credit points)
- So now D's total earned credit points becomes -> 4% of 300 points (A's credit points) + 6% of 50 points (B's credit points) + 2% of 1000 points (C's credit points)
- which is 12 + 3 + 20 = 35 final earned points (score)

Scenario 2 -

- let's say Person D, under whom Person A, B and C are working. Person A is having credit point as 300, Person B having 50 and Person C having 3000 and Person D is having 500.
- so business volume for Person D becomes = A (300) + B (50) + C (3000) + D (500)
- Business volume of Person D = 3850 credit points
- As per this credit points Person D comes under 8% slab.
- Person A is in 2% slab, Person B is in 0% slab, Person C is in 8% slab.
- Now total earned points for D is as below, 
    1. with respect to Person A -> 8% (D's slab) - 2% (A's slab) = 6% of 300 points (A's credit points)
    2. with respect to Person B -> 8% (D's slab) - 0% (B's slab) = 8% of 50 points (B's credit points)
    3. with respect to Person C -> 8% (D's slab) - 8% (C's slab) = 0% of 3000 points (C's credit points)
- So now D's total earned credit points becomes -> 6% of 300 points (A's credit points) + 8% of 50 points (B's credit points) + 0% of 3000 points (C's credit points)
- which is 18 + 4 + 0 = 22 final earned points (score)


Scenario 3 -

- let's say Person A, under whom Person B, C, D, E, F, G are working. under Person D, Person p1, p2, and p3 are working. 
- Assume for all persons (B, C, D, E, F and G) their business volume is calculated as per scenarion 1 and 2. and all their business volumn comes 1250. (For simplicity of logic, I have kept all person on same business volumn)
- as per the calculation from bottom, below are the business volumes of,
    1. Person B -> 1250 = 6% slab
    2. Person C -> 1250 = 6% slab
    3. Person D -> 1250 = 6% slab
    4. Person E -> 1250 = 6% slab
    5. Person F -> 1250 = 6% slab
    6. Person G -> 1250 = 6% slab
- Now if we calculate Person A's business volumn, it should come as below,
    1. A = A + B + C + D + E + F + G 
    2. A = 8000 = 12% slab (meaning A is in 12% slab)
- Now total earned points for A is as below,
    1. with respect to Person B -> 12% (A's slab) - 6% (B's slab) = 6% of 1250 points (B's credit points)
    2. with respect to Person C -> 12% (A's slab) - 6% (C's slab) = 6% of 1250 points (C's credit points)
    3. with respect to Person D -> 12% (A's slab) - 6% (D's slab) = 6% of 1250 points (D's credit points)
    4. with respect to Person E -> 12% (A's slab) - 6% (E's slab) = 6% of 1250 points (E's credit points)
    5. with respect to Person F -> 12% (A's slab) - 6% (F's slab) = 6% of 1250 points (F's credit points)
    6. with respect to Person G -> 12% (A's slab) - 6% (G's slab) = 6% of 1250 points (G's credit points)
- So now A's total earned credit points becomes -> 6% of 1250 points (B's credit points) + 6% of 1250 points (C's credit points) + 6% of 1250 points (D's credit points) + 6% of 1250 points (E's credit points) + 6% of 1250 points (F's credit points) + 6% of 1250 points (G's credit points)
-> which is 75 + 75 + 75 + 75 + 75 + 75 = (75 * 6) = 450 final earned points (score)

Scenario 4 - Royalty Calculation

- let's say Person P, under whom Person A, B, C, D are working.
- All 4 persons A, B, C and D croses 10,000 credit points. 
- below are the business volumn of,
    1. Person A -> 10,000 = 14% slab
    2. Person B -> 20,000 = 14% slab
    3. Person C -> 30,000 = 14% slab
    4. Person D -> 40,000 = 14% slab
- Now if we calculate Person P's business volumn, it should come as below,
    1. A + B + C + D = 1,00,000 = 14% slab
- Now total earner points for P is as below,
    1. with respect to A -> 14% (P's slab) - 14% (A's slab) = 0% of 10,000 points (A's credit points)
    2. with respect to B -> 14% (P's slab) - 14% (B's slab) = 0% of 20,000 points (B's credit points)
    3. with respect to C -> 14% (P's slab) - 14% (C's slab) = 0% of 30,000 points (C's credit points)
    4. with respect to D -> 14% (P's slab) - 14% (D's slab) = 0% of 40,000 points (D's credit points)
- since P has 0 earned points from his decendants (all are on 14% slab), hence he can now start earning royalty points from each person working under him and on 14% slab.
- Royalty for P as below
    1. with respect to A -> 1% of 10,000 points (A's credit points) = 100 points
    2. with respect to B -> 1% of 20,000 points (B's credit points) = 200 points
    3. with respect to C -> 1% of 30,000 points (C's credit points) = 300 points
    4. with respect to D -> 1% of 40,000 points (D's credit points) = 400
- So now P's total earned credit points becomes -> 1000 final earned points (score)

Scenario 5 - 

- let's say Person P, under whom Person A, B, C, D, E, F and G are working.
- Person A, B, C and D's business volumn is 10,000 credit points each respectively. and Person E's business volumn is 2000 points, Person F's business volumn is 3000 points and Person G's business volumn is 4000 points.
- below are the business volumn and slab for each person working under Person P,
    1. Person A -> 10,000 = 14% slab
    2. Person B -> 10,000 = 14% slab
    3. Person C -> 10,000 = 14% slab
    4. Person D -> 10,000 = 14% slab
    5. Person E -> 2000 = 6% slab
    6. Person F -> 3000 = 8% slab
    7. Person G -> 4000 = 8% slab
- Now if we calculate Person P's total business volumn,
    1. P = A + B + C + D + E + F + G
    2. P = 10,000 + 10,000 + 10,000 + 10,000 + 2000 + 3000 + 4000
    3. P = 49,000 points = 14% slab
- Now total earned points for Person P is as below,
    1. with respect to A -> 14% (P's slab) - 14% (A's slab) = 0% of 10,000 points (A's credit points)
    2. with respect to B -> 14% (P's slab) - 14% (B's slab) = 0% of 10,000 points (B's credit points)
    3. with respect to C -> 14% (P's slab) - 14% (C's slab) = 0% of 10,000 points (C's credit points)
    4. with respect to D -> 14% (P's slab) - 14% (D's slab) = 0% of 10,000 points (D's credit points)
    5. with respect to E -> 14% (P's slab) - 6% (E's slab) = 8% of 2000 points (E's credit points)
    6. with respect to F -> 14% (P's slab) - 8% (F's slab) = 6% of 3000 points (F's credit points)
    7. with respect to G -> 14% (P's slab) - 8% (G's slab) = 6% of 4000 points (G's credit points)
- here Person A, B, C and D comes under 14% slab, Person P will start getting royalty from this person's buisness volumn.
- Royalty of Person P's as below,
    1. with respect to A -> 1% of 10,000 points (A's credit points) = 100 points
    2. with respect to B -> 1% of 10,000 points (B's credit points) = 100 points
    3. with respect to C -> 1% of 10,000 points (C's credit points) = 100 points
    4. with respect to D -> 1% of 10,000 points (D's credit points) = 100
- so now if we calculate total earned points for Person P,
    1. 100 (Royalty from Person A) + 100 (Royalty from Person B) + 100 (Royalty from Person C) + 100 (Royalty from Person D) + 8% of 2000 points (E's credit points) + 6% of 3000 points (F's credit points) + 6% of 4000 points (G's credit points)
    2. 100 + 100 + 100 + 100 + 160 + 180 + 240
    3. total 980 earned points for Person P
- So now P's total earned credit points becomes -> 980 final earned points (score)