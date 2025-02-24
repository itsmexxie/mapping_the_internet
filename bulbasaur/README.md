# Mapping the Internet - Bulbasaur
A webpage which uses the Cubone API to display data about adresses.

## Usage
### Node
1. Install [node](https://nodejs.org/en) and (preferably) [yarn](https://yarnpkg.com/).
2. Create an .env file containing the following enviromental variables:

```
PUBLIC_API_URL="{{ URL POINTING TO YOUR INSTANCE OF CUBONE }}"
```

3. Run with either `npm run dev` or `yarn dev`.

### Docker
Build with `docker build -f bulbasaur/Dockerfile .` (must be ran from the root of this repo!). Than run the resulting image with the following command:
```
docker run --rm -v ./.env:/.env -p {{ CUSTOM }}:{{ API_PORT }} {{ IMAGE_ID }}
```

The API_PORT in the docker run command must be the same one as the port the application launches with. This should in almost
all cases be `3000`.

### Public instance
