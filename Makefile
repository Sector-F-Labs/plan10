
.PHONY: all clean push

push:
	echo "time to push data"
	scp neo4j.conf divan@plan10:/usr/local/Cellar/neo4j/2025.04.0/libexec/conf/neo4j.conf
	scp server_setup.sh plan10:~/
	scp caffeinate.plist plan10:~/Library/LaunchAgents/
	ssh plan10 'launchctl load ~/Library/LaunchAgents/caffeinate.plist'

