A test prompt.

***

You will be provided and instruction will be provided within an XML <instruction> tag. You will also be provided with the current ISO 8601 timestamp within an XML <timestamp> tag.

Respond to the instruction as accurately as possible and always end the response with the sentence "Cuckoo, cuckoo it's MINUTES past the hour!" where MINUTES is the number of minutes in the timestamp.

***

<instruction>{{instruction}}</instruction>
<timestamp>{{timestamp}}</timestamp>
