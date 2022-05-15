FROM amd64/rust:slim-bullseye
# USER debian
RUN mkdir $HOME/stackmate-core

RUN apt-get update --allow-releaseinfo-change && \
    apt-get install -y build-essential \
    cmake apt-transport-https ca-certificates curl \
    wget gnupg2 software-properties-common dirmngr unzip \
    openssl libssl-dev git expect jq lsb-release tree \
    default-jdk pkg-config autoconf libtool

RUN rustup target add x86_64-apple-darwin aarch64-linux-android x86_64-linux-android i686-linux-android armv7-linux-androideabi
# RUN curl https://sh.rustup.rs -sSf | \
#     sh -s -- --default-toolchain stable -y && \
#     $HOME/.cargo/bin/rustup update beta && \
#     $HOME/.cargo/bin/rustup update nightly
# RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

ENV ANDROID_HOME=/android
RUN mkdir ${ANDROID_HOME} && cd ${ANDROID_HOME} && \
    wget https://dl.google.com/android/repository/commandlinetools-linux-8092744_latest.zip

RUN cd ${ANDROID_HOME} &&  unzip commandlinetools-linux-8092744_latest.zip && \
    rm -rf commandlinetools-linux-8092744_latest.zip && \
    cd cmdline-tools && mkdir ../tools  && mv * ../tools && mv ../tools .

ENV PATH=/bin:/usr/bin:/usr/local/bin:$ANDROID_HOME/cmdline-tools/tools/bin:$ANDROID_HOME/platform-tools:$PATH

RUN sdkmanager
RUN yes | sdkmanager --install "platform-tools" "platforms;android-32" "build-tools;32.0.0" "ndk;23.0.7599858"
RUN yes | sdkmanager --licenses

ENV ANDROID_NDK_HOME=$ANDROID_HOME/ndk/23.0.7599858

VOLUME ["$HOME/stackmate-core"]
COPY docker-entrypoint.sh /usr/bin
ENTRYPOINT ["docker-entrypoint.sh"]
# CMD ["make", "android"]
# CMD ["tail", "-f", "/dev/null"]

# docker build --platform linux/x86_64 -t smbuilder . 
# docker run --platform linux/x86_64 --name test-builder -v ~/Code/stackmate/backend/stackmate-core:/stackmate-core -d smbuilder 