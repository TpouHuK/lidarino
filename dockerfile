FROM rustembedded/cross:arm-unknown-linux-gnueabihf

ARG DEBIAN_FRONTEND=noninteractive

# ENV TZ=Europe/Kiev
# RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
# RUN apt-get update && apt-get install --assume-yes libopencv-dev clang libclang-dev

# requirements of bindgen, see https://rust-lang.github.io/rust-bindgen/requirements.html
RUN DEBIAN_FRONTEND=noninteractive apt update
RUN DEBIAN_FRONTEND=noninteractive apt install -y llvm-dev libclang-dev clang 

# cross compile opencv, see https://docs.opencv.org/4.x/d0/d76/tutorial_arm_crosscompile_with_cmake.html
RUN DEBIAN_FRONTEND=noninteractive apt install -y git build-essential cmake

RUN git clone --depth 1 --branch '4.5.1' https://github.com/opencv/opencv.git && \
    cd opencv/platforms/linux && \
    mkdir build && \
    cd build && \
    cmake -DCMAKE_TOOLCHAIN_FILE=../arm-gnueabi.toolchain.cmake ../../.. && \
    make -j$(nproc) && \
    make install


ENV CMAKE_PREFIX_PATH="/opencv/platforms/linux/build/install"
