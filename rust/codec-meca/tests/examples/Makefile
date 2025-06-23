all: \
	cb91eca4-74d6-1014-98bf-f67ca965a351.meca \
	f2af58f0-7111-1014-b4ee-f306f628684d.meca \
	ffa37796-6caf-1014-bed2-8e51ecdb86df.meca

# The following are relatively small examples chosen haphazardly from
#  aws s3 ls --human-readable --request-payer requester s3://biorxiv-src-monthly/Current_Content/June_2025/

cb91eca4-74d6-1014-98bf-f67ca965a351.meca:
	aws s3 cp --request-payer requester s3://biorxiv-src-monthly/Current_Content/June_2025/$@ $@

f2af58f0-7111-1014-b4ee-f306f628684d.meca:
	aws s3 cp --request-payer requester s3://biorxiv-src-monthly/Current_Content/June_2025/$@ $@

ffa37796-6caf-1014-bed2-8e51ecdb86df.meca:
	aws s3 cp --request-payer requester s3://biorxiv-src-monthly/Current_Content/June_2025/$@ $@
