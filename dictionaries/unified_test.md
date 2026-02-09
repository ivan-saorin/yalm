# unified_test — Phase 18 Five-Space Integration Test
# 50 questions across CONTENT, MATH, GRAMMAR, TASK, SELF, cross-space, and pipeline

---

## Group 1: CONTENT only (from dict5_test Q01-Q10)

**Q01**: Is a dog an animal?
**A**: Yes

**Q02**: Is the sun hot?
**A**: Yes

**Q03**: Is a cat small?
**A**: Yes

**Q04**: Can a dog make sound?
**A**: Yes

**Q05**: Can a person make things?
**A**: Yes

**Q06**: Is a dog a thing?
**A**: Yes

**Q07**: Is a cat a thing?
**A**: Yes

**Q08**: Can a dog eat?
**A**: Yes

**Q09**: Can a cat feel?
**A**: Yes

**Q10**: Can a dog live in a place?
**A**: Yes

---

## Group 2: MATH only (from multispace_test Q01-Q05)

**Q11**: What is two plus three?
**A**: five

**Q12**: Is five a number?
**A**: Yes

**Q13**: Is three more than one?
**A**: Yes

**Q14**: What is five minus two?
**A**: three

**Q15**: What comes after four?
**A**: five

---

## Group 3: GRAMMAR only (from multispace_test Q06-Q10)

**Q16**: Is dog a noun?
**A**: Yes

**Q17**: Is eat a verb?
**A**: Yes

**Q18**: What is a sentence?
**A**: words in order that tell a thing

**Q19**: Is big a property?
**A**: Yes

**Q20**: What is a subject?
**A**: the thing in a sentence that does the action

---

## Group 4: TASK routing with CONTENT

**Q21**: Is "is a dog an animal" a content task?
**A**: Yes

**Q22**: Is "what is two plus three" a content task?
**A**: No

**Q23**: What kind of task is "is the sun hot"?
**A**: a content task

**Q24**: What kind of task is "is dog a noun"?
**A**: a word task

**Q25**: What kind of task is "how many animals"?
**A**: a number task

---

## Group 5: Cross-space with CONTENT

**Q26**: Is dog a noun?
**A**: Yes
Note: CONTENT + GRAMMAR (dog in CONTENT, noun in GRAMMAR)

**Q27**: Is eat a verb?
**A**: Yes
Note: CONTENT + GRAMMAR

**Q28**: Is hot a property?
**A**: Yes
Note: CONTENT + GRAMMAR

**Q29**: Is the sun a thing?
**A**: Yes
Note: CONTENT (direct, within single space)

**Q30**: Is a dog an animal and a thing?
**A**: Yes
Note: CONTENT (boolean, within single space)

**Q31**: Can an animal eat? Write the answer as a sentence.
**A**: an animal can eat
Note: CONTENT + GRAMMAR

**Q32**: Is three a noun?
**A**: Yes
Note: MATH + GRAMMAR

**Q33**: Is the sun big? Is five big?
**A**: Yes and Yes
Note: CONTENT + MATH

**Q34**: What is a dog?
**A**: an animal
Note: CONTENT

**Q35**: Is ball a noun or a verb?
**A**: a noun
Note: CONTENT + GRAMMAR

---

## Group 6: Full pipeline

**Q36**: Two plus three. The answer is a number. Is it big?
**A**: No
Note: TASK -> MATH + CONTENT (five is not big in content terms)

**Q37**: What is a cat? Is cat a noun?
**A**: an animal. Yes.
Note: CONTENT + GRAMMAR (compound, multi-answer)

**Q38**: Can a person make a sound? Write a sentence.
**A**: a person can make a sound
Note: CONTENT + GRAMMAR

**Q39**: Is eat an action and a verb?
**A**: Yes
Note: CONTENT + GRAMMAR

**Q40**: One plus one. Is the result equal to two? Answer with yes or no.
**A**: Yes
Note: TASK -> MATH

---

## Group 7: SELF Identity

**Q41**: What are you?
**A**: a thing that reads words and learns
Note: SELF space, direct definition lookup

**Q42**: Are you a person?
**A**: No
Note: SELF space, dafhne definition says "not a person"

**Q43**: Are you an animal?
**A**: No
Note: SELF space, dafhne definition says "not an animal"

**Q44**: Can you make mistakes?
**A**: Yes
Note: SELF space, mistake is defined as something dafhne can make

**Q45**: Do you have a body?
**A**: No
Note: SELF space, dafhne has no body

---

## Group 8: SELF Capabilities

**Q46**: Can you count?
**A**: Yes
Note: SELF knows dafhne can count, bridges to MATH

**Q47**: Can you see?
**A**: No
Note: SELF knows dafhne cannot see (no eyes)

**Q48**: Can you read?
**A**: Yes
Note: SELF direct capability

**Q49**: Can you eat?
**A**: No
Note: SELF knows dafhne cannot eat (no body)

**Q50**: Do you know what a dog is?
**A**: Yes
Note: SELF + CONTENT meta-check (dog exists in CONTENT space)
