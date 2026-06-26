#! /bin/zsh
container system start 
softwareupdate --install-rosetta --agree-to-license

container run -d --name mssql --arch amd64 --memory 4g --cpus 2 -e ACCEPT_EULA=Y -e MSSQL_SA_PASSWORD='YourStrong!Passw0rd' -p 1433:1433 mcr.microsoft.com/mssql/server:2022-latest
container start mssql 