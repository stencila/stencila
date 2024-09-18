# Update the src/ai.robots.txt file by fetching it
# from https://github.com/ai-robots-txt/ai.robots.txt
ai.robots.txt:
	curl -o src/ai.robots.txt https://raw.githubusercontent.com/ai-robots-txt/ai.robots.txt/main/robots.txt
	sed -i '1iUser-agent: ALL-AI-BOTS' src/ai.robots.txt
.PHONY: ai.robots.txt
