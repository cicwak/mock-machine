FROM node:22-alpine
WORKDIR /app

COPY apps/frontend/package.json ./
COPY apps/frontend/package-lock.json ./
RUN npm ci

COPY apps/frontend ./

EXPOSE 5173
CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]
